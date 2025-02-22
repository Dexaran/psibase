#pragma once

#include <psibase/ForkDb.hpp>
#include <psibase/net_base.hpp>
#include <psibase/random_timer.hpp>
#include <psio/reflect.hpp>

#include <algorithm>
#include <cassert>
#include <chrono>
#include <cstdint>
#include <cstring>
#include <memory>
#include <variant>
#include <vector>

#include <iostream>

namespace psibase::net
{
#ifndef PSIO_REFLECT_INLINE
#define PSIO_REFLECT_INLINE(name, ...)                      \
   PSIO_REFLECT(name, __VA_ARGS__)                          \
   friend reflect_impl_##name get_reflect_impl(const name&) \
   {                                                        \
      return {};                                            \
   }
#endif

   // round tp to the nearest multiple of d towards negative infinity
   template <typename Clock, typename Duration1, typename Duration2>
   std::chrono::time_point<Clock, Duration1> floor2(std::chrono::time_point<Clock, Duration1> tp,
                                                    Duration2                                 d)
   {
      Duration1 d1{d};
      auto      rem = tp.time_since_epoch() % d1;
      if (rem < Duration1{})
      {
         rem += d1;
      }
      return tp - rem;
   }

   // This protocol is based on RAFT, with some simplifications.
   // i.e. the blockchain structure is sufficient to guarantee log matching.
   template <typename Derived, typename Timer>
   struct basic_cft_consensus
   {
      using term_id                              = std::uint64_t;
      using block_num                            = std::uint32_t;
      using id_type                              = int;
      static constexpr producer_id null_producer = {};

      auto& network() { return static_cast<Derived*>(this)->network(); }
      auto& chain() { return static_cast<Derived*>(this)->chain(); }

      enum class producer_state : std::uint8_t
      {
         unknown,
         follower,
         candidate,
         leader,
         nonvoting
      };

      struct hello_request
      {
         static constexpr unsigned type = 32;
         ExtendedBlockId           xid;
         std::string               to_string() const
         {
            return "hello: id=" + loggers::to_string(xid.id()) +
                   " blocknum=" + std::to_string(xid.num());
         }
         PSIO_REFLECT_INLINE(hello_request, xid)
      };

      struct hello_response
      {
         static constexpr unsigned type  = 33;
         char                      dummy = 0;
         std::string               to_string() const { return "hello response"; }
         PSIO_REFLECT_INLINE(hello_response, dummy)
      };

      struct peer_connection
      {
         explicit peer_connection(peer_id id) : id(id) {}
         ~peer_connection() { std::memset(this, 0xCC, sizeof(*this)); }
         ExtendedBlockId last_sent;
         ExtendedBlockId last_received;
         bool            syncing = false;
         peer_id         id;
         bool            ready  = false;
         bool            closed = false;
         // True once we have received a hello_response from the peer
         bool peer_ready = false;
         // TODO: we may be able to save some space, because last_received is
         // not meaningful until we're finished with hello.
         // The most recent hello message sent or the next queued hello message
         hello_request hello;
         bool          hello_sent;
      };

      template <typename ExecutionContext>
      explicit basic_cft_consensus(ExecutionContext& ctx) : _election_timer(ctx), _block_timer(ctx)
      {
         logger.add_attribute("Channel",
                              boost::log::attributes::constant(std::string("consensus")));
      }

      producer_id                  self = null_producer;
      std::shared_ptr<ProducerSet> active_producers[2];
      term_id                      current_term = 0;
      producer_id                  voted_for    = null_producer;
      std::vector<producer_id>     votes_for_me[2];

      block_num last_applied = 0;

      producer_state            _state = producer_state::unknown;
      basic_random_timer<Timer> _election_timer;
      Timer                     _block_timer;
      std::chrono::milliseconds _timeout        = std::chrono::seconds(3);
      std::chrono::milliseconds _block_interval = std::chrono::seconds(1);

      std::vector<block_num>                        match_index[2];
      std::vector<std::unique_ptr<peer_connection>> _peers;

      loggers::common_logger logger;

      struct append_entries_request
      {
         static constexpr unsigned          type = 34;
         psio::shared_view_ptr<SignedBlock> block;
         PSIO_REFLECT_INLINE(append_entries_request, block)
         std::string to_string() const
         {
            BlockInfo info{*block->block()};
            return "append_entries: term=" +
                   std::to_string(term_id{block->block()->header()->term()}) +
                   " leader=" + AccountNumber{block->block()->header()->producer()}.str() +
                   " id=" + loggers::to_string(info.blockId) +
                   " blocknum=" + std::to_string(BlockNum{block->block()->header()->blockNum()}) +
                   " irreversible=" +
                   std::to_string(BlockNum{block->block()->header()->commitNum()});
         }
      };

      struct append_entries_response
      {
         static constexpr unsigned type = 35;
         term_id                   term;
         producer_id               follower_id;
         BlockNum                  head_num;
         Claim                     claim;

         Claim signer() const { return claim; }
         PSIO_REFLECT_INLINE(append_entries_response, term, follower_id, head_num, claim);
         std::string to_string() const
         {
            return "append_entries response: term=" + std::to_string(term) +
                   " follower=" + follower_id.str() + " blocknum=" + std::to_string(head_num);
         }
      };

      struct request_vote_request
      {
         static constexpr unsigned type = 36;
         term_id                   term;
         producer_id               candidate_id;
         block_num                 last_log_index;
         term_id                   last_log_term;
         Claim                     claim;

         Claim signer() const { return claim; }
         PSIO_REFLECT_INLINE(request_vote_request,
                             term,
                             candidate_id,
                             last_log_index,
                             last_log_term,
                             claim)
         std::string to_string() const
         {
            return "request_vote: term=" + std::to_string(term) +
                   " candidate=" + candidate_id.str();
         }
      };
      struct request_vote_response
      {
         static constexpr unsigned type = 37;
         term_id                   term;
         producer_id               candidate_id;
         producer_id               voter_id;
         bool                      vote_granted;
         Claim                     claim;

         Claim signer() const { return claim; }
         PSIO_REFLECT_INLINE(request_vote_response,
                             term,
                             candidate_id,
                             voter_id,
                             vote_granted,
                             claim)
         std::string to_string() const
         {
            return "vote: term=" + std::to_string(term) + " candidate=" + candidate_id.str() +
                   " voter=" + voter_id.str() + " vote granted=" + std::to_string(vote_granted);
         }
      };

      using message_type = std::variant<hello_request,
                                        hello_response,
                                        append_entries_request,
                                        append_entries_response,
                                        request_vote_request,
                                        request_vote_response>;

      peer_connection& get_connection(peer_id id)
      {
         for (const auto& peer : _peers)
         {
            if (peer->id == id)
            {
               return *peer;
            }
         }
         assert(!"Unknown peer connection");
      }

      void disconnect(peer_id id)
      {
         auto pos =
             std::find_if(_peers.begin(), _peers.end(), [&](const auto& p) { return p->id == id; });
         assert(pos != _peers.end());
         if ((*pos)->syncing)
         {
            (*pos)->closed = true;
         }
         else
         {
            _peers.erase(pos);
         }
      }

      void connect(peer_id id)
      {
         _peers.push_back(std::make_unique<peer_connection>(id));
         peer_connection& connection = get_connection(id);
         connection.hello_sent       = false;
         connection.hello.xid        = chain().get_head_state()->xid();
         async_send_hello(connection);
      }
      void async_send_hello(peer_connection& connection)
      {
         if (connection.hello_sent)
         {
            auto b = chain().get(connection.hello.xid.id());
            if (!b)
            {
               return;
            }
            auto prev = chain().get(Checksum256(b->block()->header()->previous()));
            if (prev)
            {
               connection.hello = {Checksum256(b->block()->header()->previous()),
                                   BlockNum(b->block()->header()->blockNum()) - 1};
            }
            else
            {
               // TODO: detect the case where the two nodes have no blocks in common
               // This could happen when an out-dated node tries to sync with a node
               // with a trimmed block log, for instance.
               return;
            }
         }
         connection.hello_sent = true;
         network().async_send_block(connection.id, connection.hello,
                                    [this, &connection](const std::error_code& ec)
                                    {
                                       if (!connection.peer_ready)
                                       {
                                          // TODO: rate limit hellos, delay second hello until we have received the first peer hello
                                          async_send_hello(connection);
                                       }
                                    });
      }
      void recv(peer_id origin, const hello_request& request)
      {
         auto& connection = get_connection(origin);
         if (connection.ready)
         {
            return;
         }
         if (!connection.peer_ready &&
             connection.hello.xid.num() > request.xid.num() + connection.hello_sent)
         {
            // TODO: if the block num is not known, then we've failed to find a common ancestor
            // so bail (only possible with a truncated block log)
            connection.hello.xid  = {chain().get_block_id(request.xid.num()), request.xid.num()};
            connection.hello_sent = false;
         }
         if (request.xid.id() == Checksum256{})
         {
            // sync from genesis
            connection.last_received = {Checksum256{}, 1};
            connection.last_sent     = connection.last_received;
         }
         else
         {
            if (auto b = chain().get(request.xid.id()))
            {
               // Ensure that the block number is accurate.  I have not worked out
               // what happens if the peer lies, but at least this way we guarantee
               // that our local invariants hold.
               connection.last_received = {request.xid.id(),
                                           BlockNum(b->block()->header()->blockNum())};
            }
            else if (auto* b = chain().get_state(request.xid.id()))
            {
               connection.last_received = {request.xid.id(), b->blockNum()};
            }
            else
            {
               return;
            }
            connection.last_sent = chain().get_common_ancestor(connection.last_received);
         }
         // async_send_fork will reset syncing if there is nothing to sync
         connection.syncing = true;
         connection.ready   = true;
         //std::cout << "ready: received=" << to_string(connection.last_received.id())
         //          << " common=" << to_string(connection.last_sent.id()) << std::endl;
         // FIXME: blocks and hellos need to be sequenced correctly
         network().async_send_block(connection.id, hello_response{},
                                    [this, &connection](const std::error_code&)
                                    { async_send_fork(connection); });
      }
      void recv(peer_id origin, const hello_response&)
      {
         auto& connection      = get_connection(origin);
         connection.peer_ready = true;
      }
      void set_producer_id(producer_id prod)
      {
         if (self != prod)
         {
            self = prod;
            // Re-evaluates producer state
            // This may leave the leader running even it it changed names.
            // I believe that this is safe because the fact that it's
            // still the same node guarantees that the hand-off is atomic.
            if (active_producers[0])
            {
               set_producers({active_producers[0], active_producers[1]});
            }
         }
      }
      void set_producers(
          std::pair<std::shared_ptr<ProducerSet>, std::shared_ptr<ProducerSet>> prods)
      {
         if (_state == producer_state::leader)
         {
            // match_index
            if (*active_producers[0] != *prods.first)
            {
               if (active_producers[1] && *active_producers[1] == *prods.first)
               {
                  // Leaving joint consensus
                  match_index[0] = match_index[1];
               }
               else
               {
                  match_index[0].clear();
                  match_index[0].resize(prods.first->size());
               }
            }
            // Second match_index is only active during joint consensus
            if (!prods.second)
            {
               match_index[1].clear();
            }
            else if (!active_producers[1] || *active_producers[1] != *prods.second)
            {
               match_index[1].clear();
               match_index[1].resize(prods.second->size());
            }
         }
         active_producers[0] = std::move(prods.first);
         active_producers[1] = std::move(prods.second);
         if (is_producer())
         {
            if (_state == producer_state::nonvoting || _state == producer_state::unknown)
            {
               PSIBASE_LOG(logger, info) << "Node is active producer";
               _state = producer_state::follower;
               randomize_timer();
            }
         }
         else
         {
            if (_state != producer_state::nonvoting)
            {
               PSIBASE_LOG(logger, info) << "Node is non-voting";
               _election_timer.cancel();
               stop_leader();
            }
            _state = producer_state::nonvoting;
         }
      }
      void load_producers()
      {
         current_term = chain().get_head()->term;
         set_producers(chain().getProducers());
      }
      bool is_sole_producer() const
      {
         return ((active_producers[0]->size() == 0 && self != AccountNumber()) ||
                 (active_producers[0]->size() == 1 && active_producers[0]->isProducer(self))) &&
                !active_producers[1];
      }
      bool is_producer() const
      {
         // If there are no producers set on chain, then any
         // producer can produce a block
         return (active_producers[0]->size() == 0 && self != AccountNumber() &&
                 !active_producers[1]) ||
                active_producers[0]->isProducer(self) ||
                (active_producers[1] && active_producers[1]->isProducer(self));
      }
      producer_id producer_name() const { return self; }

      void validate_producer(producer_id producer, const Claim& claim)
      {
         auto expected0 = active_producers[0]->getClaim(producer);
         auto expected1 =
             active_producers[1] ? active_producers[1]->getClaim(producer) : decltype(expected0)();
         if (!expected0 && !expected1)
         {
            throw std::runtime_error(producer.str() + " is not an active producer");
         }
         else if (claim != *expected0 && claim != *expected1)
         {
            throw std::runtime_error("Wrong key for " + producer.str());
         }
      }

      template <typename F>
      void for_each_key(F&& f)
      {
         auto claim0 = active_producers[0]->getClaim(self);
         if (claim0)
         {
            f(*claim0);
         }
         if (active_producers[1])
         {
            auto claim1 = active_producers[1]->getClaim(self);
            if (claim1 && claim0 != claim1)
            {
               f(*claim1);
            }
         }
      }

      // ---------------- block production loop --------------------------

      void start_leader()
      {
         assert(_state == producer_state::leader);
         auto head = chain().get_head();
         // The next block production time is the later of:
         // - The last block interval boundary before the current time
         // - The head block time + the block interval
         //
         auto head_time   = typename Timer::time_point{std::chrono::seconds(head->time.seconds)};
         auto block_start = std::max(head_time + _block_interval,
                                     floor2(Timer::clock_type::now(), _block_interval));
         _block_timer.expires_at(block_start + _block_interval);
         // TODO: consensus should be responsible for most of the block header
         auto commit_index = is_sole_producer() ? head->blockNum + 1 : chain().commit_index();
         chain().start_block(
             TimePointSec{static_cast<uint32_t>(
                 duration_cast<std::chrono::seconds>(block_start.time_since_epoch()).count())},
             self, current_term, commit_index);
         _block_timer.async_wait(
             [this](const std::error_code& ec)
             {
                if (ec)
                {
                   PSIBASE_LOG(logger, info) << "Stopping block production";
                   chain().abort_block();
                }
                else if (_state == producer_state::leader)
                {
                   if (auto* b = chain().finish_block())
                   {
                      update_match_index(self, b->blockNum());
                      on_fork_switch(&b->info.header);
                      chain().gc();
                   }
                   // finish_block might convert us to nonvoting
                   if (_state == producer_state::leader)
                   {
                      start_leader();
                   }
                }
             });
      }

      void stop_leader()
      {
         if (_state == producer_state::leader)
         {
            _block_timer.cancel();
         }
      }

      // The block broadcast algorithm is most independent of consensus.
      // The primary potential variation is whether a block is forwarded
      // before or after validation.

      // This should probably NOT be part of the consensus class, because
      // it's mostly invariant across consensus algorithms.
      // invariants: if the head block is not the last sent block, then
      //             exactly one instance of async_send_fork is active
      void async_send_fork(auto& peer)
      {
         if (peer.closed)
         {
            peer.syncing = false;
            disconnect(peer.id);
            return;
         }
         if (peer.last_sent.num() != chain().get_head()->blockNum)
         {
            auto next_block_id = chain().get_block_id(peer.last_sent.num() + 1);
            peer.last_sent     = {next_block_id, peer.last_sent.num() + 1};
            auto next_block    = chain().get(next_block_id);

            network().async_send_block(peer.id, append_entries_request{next_block},
                                       [this, &peer](const std::error_code& e)
                                       { async_send_fork(peer); });
         }
         else
         {
            peer.syncing = false;
         }
      }
      // This should be run whenever there is a new head block on the local chain
      // Note: this needs to run before orphaned blocks in the old fork
      // are pruned.  It must remove references to blocks that are not
      // part of the new best fork.
      void on_fork_switch(BlockHeader* new_head)
      {
         if (_state == producer_state::follower && new_head->term == current_term &&
             new_head->blockNum > chain().commit_index())
         {
            for_each_key(
                [&](const auto& k)
                {
                   network().sendto(
                       new_head->producer,
                       append_entries_response{current_term, self, new_head->blockNum, k});
                });
         }
         // TODO: how do we handle a fork switch during connection startup?
         for (auto& peer : _peers)
         {
            // ---------- TODO: dispatch to peer connection strand -------------
            if (!peer->peer_ready)
            {
               auto new_id = chain().get_common_ancestor(peer->hello.xid);
               if (peer->hello.xid != new_id)
               {
                  peer->hello.xid  = new_id;
                  peer->hello_sent = false;
               }
            }
            // if last sent block is after committed, back it up to the nearest block in the chain
            if (peer->ready)
            {
               // Note: Checking best_received primarily prevents received blocks
               // from being echoed back to their origin.
               auto best_received = chain().get_common_ancestor(peer->last_received);
               auto best_sent     = chain().get_common_ancestor(peer->last_sent);
               peer->last_sent = best_received.num() > best_sent.num() ? best_received : best_sent;
               // if the peer is synced, start async_send_fork
               if (!peer->syncing)
               {
                  peer->syncing = true;
                  async_send_fork(*peer);
               }
            }
            // ------------------------------------------------------------------
         }
      }

      void on_recv_block(auto& peer, const ExtendedBlockId& xid)
      {
         peer.last_received = xid;
         if (chain().in_best_chain(xid) && xid.num() > peer.last_sent.num())
         {
            peer.last_sent = xid;
         }
      }

      // ------------------- Implementation utilities ----- -----------
      void update_match_index(producer_id producer, block_num match)
      {
         auto jointCommitIndex = std::min(update_match_index(producer, match, 0),
                                          update_match_index(producer, match, 1));
         if (jointCommitIndex == std::numeric_limits<BlockNum>::max())
         {
            assert(active_producers[0]->size() == 0);
            assert(!active_producers[1]);
            assert(producer == self);
            jointCommitIndex = match;
         }
         if (chain().commit(match))
         {
            set_producers(chain().getProducers());
         }
      }
      BlockNum update_match_index(producer_id producer, BlockNum confirmed, std::size_t group)
      {
         if (active_producers[group])
         {
            if (auto idx = active_producers[group]->getIndex(producer))
            {
               match_index[group][*idx] = std::max(match_index[group][*idx], confirmed);
            }
            if (!match_index[group].empty())
            {
               std::vector<block_num> match = match_index[group];
               auto                   mid   = match.size() / 2;
               std::nth_element(match.begin(), match.begin() + mid, match.end());
               return match[mid];
            }
         }
         return std::numeric_limits<BlockNum>::max();
      }
      // This should always run first when handling any message
      void update_term(term_id term)
      {
         if (term > current_term)
         {
            if (_state == producer_state::leader)
            {
               match_index[0].clear();
               match_index[1].clear();
               stop_leader();
               randomize_timer();
            }
            votes_for_me[0].clear();
            votes_for_me[1].clear();
            current_term = term;
            voted_for    = null_producer;
            _state       = producer_state::follower;
         }
      }
      void check_votes()
      {
         if (votes_for_me[0].size() > active_producers[0]->size() / 2 &&
             (!active_producers[1] || votes_for_me[1].size() > active_producers[1]->size() / 2))
         {
            _state = producer_state::leader;
            match_index[0].clear();
            match_index[0].resize(active_producers[0]->size());
            if (active_producers[1])
            {
               match_index[1].clear();
               match_index[1].resize(active_producers[1]->size());
            }
            PSIBASE_LOG(logger, info)
                << "Starting block production for term " << current_term << " as " << self.str();
            start_leader();
         }
      }
      // -------------- The timer loop --------------------
      void randomize_timer()
      {
         // Don't bother waiting if we're the only producer
         if (active_producers[0]->size() <= 1 && !active_producers[1])
         {
            if (_state == producer_state::follower)
            {
               request_vote();
            }
         }
         else
         {
            _election_timer.expires_after(_timeout, _timeout * 2);
            _election_timer.async_wait(
                [this](const std::error_code& ec)
                {
                   if (!ec &&
                       (_state == producer_state::follower || _state == producer_state::candidate))
                   {
                      PSIBASE_LOG(logger, info)
                          << "Timeout: Starting leader election for term " << (current_term + 1);
                      request_vote();
                   }
                });
         }
      }
      void request_vote()
      {
         ++current_term;
         voted_for = self;
         votes_for_me[0].clear();
         if (active_producers[0]->isProducer(self) || active_producers[0]->size() == 0)
         {
            votes_for_me[0].push_back(self);
         }
         if (active_producers[1] && active_producers[1]->isProducer(self))
         {
            votes_for_me[1].push_back(self);
         }
         _state = producer_state::candidate;
         randomize_timer();
         for_each_key(
             [&](const auto& k)
             {
                network().multicast_producers(request_vote_request{
                    current_term, self, chain().get_head()->blockNum, chain().get_head()->term, k});
             });
         check_votes();
      }
      // ----------- handling of incoming messages -------------
      void recv(peer_id origin, const append_entries_request& request)
      {
         auto& connection = get_connection(origin);
         auto  term       = BlockNum{request.block->block()->header()->term()};
         update_term(term);
         if (term >= current_term)
         {
            _election_timer.restart();
         }
         // TODO: should the leader ever accept a block from another source?
         if (chain().insert(request.block))
         {
            BlockInfo       info{*request.block->block()};
            ExtendedBlockId xid = {info.blockId, info.header.blockNum};
            on_recv_block(connection, xid);
            //std::cout << "recv node=" << self.str() << " id=" << to_string(xid.id()) << std::endl;
            chain().async_switch_fork(
                [this](BlockHeader* h)
                {
                   if (chain().commit(h->commitNum))
                   {
                      set_producers(chain().getProducers());
                   }
                   on_fork_switch(h);
                   chain().gc();
                });
         }
      }
      void recv(peer_id, const append_entries_response& response)
      {
         validate_producer(response.follower_id, response.claim);
         update_term(response.term);
         if (response.term == current_term)
         {
            if (_state == producer_state::leader)
            {
               update_match_index(response.follower_id, response.head_num);
            }
         }
         // otherwise ignore out-dated response
      }
      void recv(peer_id, const request_vote_request& request)
      {
         validate_producer(request.candidate_id, request.claim);
         update_term(request.term);
         bool vote_granted = false;
         // Can we vote for this candidate?
         if (_state == producer_state::follower)
         {
            if (request.term >= current_term &&
                (voted_for == null_producer || voted_for == request.candidate_id))
            {
               // Is the candidate up-to-date?
               auto head = chain().get_head();
               if (request.last_log_term > head->term || (request.last_log_term == head->term &&
                                                          request.last_log_index >= head->blockNum))
               {
                  _election_timer.restart();
                  vote_granted = true;
                  voted_for    = request.candidate_id;
               }
            }
            for_each_key(
                [&](const auto& k)
                {
                   network().sendto(request.candidate_id,
                                    request_vote_response{.term         = current_term,
                                                          .candidate_id = request.candidate_id,
                                                          .voter_id     = self,
                                                          .vote_granted = vote_granted,
                                                          .claim        = k});
                });
         }
      }
      void recv(peer_id, const request_vote_response& response)
      {
         validate_producer(response.voter_id, response.claim);
         update_term(response.term);
         if (response.candidate_id == self && response.term == current_term &&
             response.vote_granted && _state == producer_state::candidate)
         {
            for (auto i : {0, 1})
            {
               auto& votes = votes_for_me[i];
               if (active_producers[i] && active_producers[i]->isProducer(response.voter_id) &&
                   std::find(votes.begin(), votes.end(), response.voter_id) == votes.end())
               {
                  votes.push_back(response.voter_id);
               }
            }
            check_votes();
         }
      }
   };

}  // namespace psibase::net

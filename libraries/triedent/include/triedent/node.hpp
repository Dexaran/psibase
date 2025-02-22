#pragma once
#include <triedent/debug.hpp>
#include <triedent/ring_alloc.hpp>

namespace triedent
{
   inline constexpr bool debug_nodes = false;

   using key_view   = std::string_view;
   using value_view = std::string_view;
   using key_type   = std::string;
   using value_type = key_type;

   object_id bump_refcount_or_copy(ring_allocator& ra, object_id id);

   class node
   {
     public:
      inline uint8_t key_size() const { return (*reinterpret_cast<const uint8_t*>(this)); }
   };

   class value_node : public node
   {
     public:
      inline uint32_t    key_size() const { return _key_size; }
      inline char*       key_ptr() { return ((char*)this) + sizeof(value_node); }
      inline const char* key_ptr() const { return ((const char*)this) + sizeof(value_node); }
      inline uint32_t    data_size() const
      {
         return (reinterpret_cast<const object_header*>(this) - 1)->size - key_size() -
                sizeof(value_node);
         //return sizes >> 8;
      }
      inline char*       data_ptr() { return ((char*)this) + sizeof(value_node) + key_size(); }
      inline const char* data_ptr() const
      {
         return ((const char*)this) + sizeof(value_node) + key_size();
      }

      inline object_id*       roots() { return reinterpret_cast<object_id*>(data_ptr()); }
      inline const object_id* roots() const
      {
         return reinterpret_cast<const object_id*>(data_ptr());
      }
      auto num_roots() const { return data_size() / sizeof(object_id); }

      inline value_view data() const { return value_view(data_ptr(), data_size()); }
      inline key_view   key() const { return key_view(key_ptr(), key_size()); }

      inline static std::pair<location_lock, value_node*> make(ring_allocator& a,
                                                               key_view        key,
                                                               value_view      val,
                                                               node_type       type,
                                                               bool            bump_root_refs)
      {
         assert(val.size() < 0xffffff - key.size() - sizeof(value_node));
         uint32_t alloc_size = sizeof(value_node) + key.size() + val.size();
         auto     r          = a.alloc(alloc_size, type);
         if constexpr (debug_nodes)
            std::cout << r.first.get_id().id << ": construct value_node: type=" << (int)type
                      << std::endl;
         return std::make_pair(std::move(r.first),
                               new (r.second) value_node(a, key, val, bump_root_refs));
      }

     private:
      value_node(ring_allocator& ra, key_view key, value_view val, bool bump_root_refs)
      {
         _key_size = key.size();
         memcpy(key_ptr(), key.data(), key_size());
         if (bump_root_refs)
         {
            if constexpr (debug_nodes)
            {
               std::cout << "value_node(): key_size=" << (int)_key_size
                         << " data_size=" << (int)val.size()
                         << " bump roots=" << val.size() / sizeof(object_id) << "\n    ";
               auto n   = val.size() / sizeof(object_id);
               auto src = reinterpret_cast<const object_id*>(val.data());
               while (n--)
                  std::cout << src++->id << " ";
               std::cout << std::endl;
            }
            assert(val.size() % sizeof(object_id) == 0);
            auto n    = val.size() / sizeof(object_id);
            auto src  = reinterpret_cast<const object_id*>(val.data());
            auto dest = roots();
            while (n--)
               *dest++ = bump_refcount_or_copy(ra, *src++);
         }
         else
         {
            if constexpr (debug_nodes)
               std::cout << "value_node(): key_size=" << (int)_key_size
                         << " data_size=" << val.size() << std::endl;
            memcpy(data_ptr(), val.data(), val.size());
         }
      }
      //  uint32_t sizes;
      uint8_t _key_size;
   };
   static_assert(sizeof(value_node) == 1, "unexpected padding");

   class inner_node : public node
   {
     public:
      inline object_id&       branch(uint8_t b);
      inline const object_id& branch(uint8_t b) const;
      inline object_id        value() const { return _value; }
      inline object_id&       value() { return _value; }
      inline void             set_value(object_id i) { _value = i; }

      inline uint32_t num_children() const { return num_branches() + (_value.id != 0); }
      inline uint32_t num_branches() const { return std::popcount(_present_bits); }
      inline uint64_t branches() const { return _present_bits; }

      template <typename... Ts>
      inline static uint64_t branches(Ts... bit_num)
      {
         return ((1ull << bit_num) | ...);
      }

      inline uint8_t lower_bound(uint8_t b) const;
      inline int8_t  reverse_lower_bound(uint8_t b) const;
      inline uint8_t upper_bound(uint8_t b) const;

      inline static std::pair<location_lock, inner_node*> make(ring_allocator&   a,
                                                               const inner_node& in,
                                                               key_view          prefix,
                                                               object_id         val,
                                                               uint64_t          branches);

      inline static std::pair<location_lock, inner_node*> make(ring_allocator& a,
                                                               key_view        prefix,
                                                               object_id       val,
                                                               uint64_t        branches);

      inline bool has_branch(uint32_t b) const { return _present_bits & (1ull << b); }

      inline key_view key() const { return key_view(key_ptr(), key_size()); }

      inline int32_t     branch_index(uint32_t branch) const;
      object_id*         children() { return reinterpret_cast<object_id*>(this + 1); }
      const object_id*   children() const { return reinterpret_cast<const object_id*>(this + 1); }
      inline char*       key_ptr() { return reinterpret_cast<char*>(children() + num_branches()); }
      inline const char* key_ptr() const
      {
         return reinterpret_cast<const char*>(children() + num_branches());
      }

     private:
      inner_node(object_id         id,
                 ring_allocator&   a,
                 const inner_node& in,
                 key_view          prefix,
                 object_id         val,
                 uint64_t          branches);
      inner_node(object_id id, key_view prefix, object_id val, uint64_t branches);

      uint8_t   _prefix_length = 0;  // mirrors value nodes to signal type and prefix length
      uint8_t   _reserved_a    = 0;  // future use
      uint8_t   _reserved_b    = 0;  // future use
      object_id _value;              // this is 5 bytes
      uint64_t  _present_bits = 0;   // keep this 8 byte aligned for popcount instructions
   } __attribute__((packed));
   static_assert(sizeof(inner_node) == 3 + 5 + 8, "unexpected padding");

   inline std::pair<location_lock, inner_node*> inner_node::make(ring_allocator&   a,
                                                                 const inner_node& in,
                                                                 key_view          prefix,
                                                                 object_id         val,
                                                                 uint64_t          branches)
   {
      uint32_t alloc_size =
          sizeof(inner_node) + prefix.size() + std::popcount(branches) * sizeof(object_id);
      auto p  = a.alloc(alloc_size, node_type::inner);
      auto id = p.first.get_id();
      if constexpr (debug_nodes)
         std::cout << id.id << ": construct inner_node" << std::endl;
      return std::make_pair(std::move(p.first),
                            new (p.second) inner_node(id, a, in, prefix, val, branches));
   }

   inline std::pair<location_lock, inner_node*> inner_node::make(ring_allocator& a,
                                                                 key_view        prefix,
                                                                 object_id       val,
                                                                 uint64_t        branches)
   {
      uint32_t alloc_size =
          sizeof(inner_node) + prefix.size() + std::popcount(branches) * sizeof(object_id);
      auto p  = a.alloc(alloc_size, node_type::inner);
      auto id = p.first.get_id();
      if constexpr (debug_nodes)
         std::cout << id.id << ": construct inner_node" << std::endl;
      return std::make_pair(std::move(p.first),
                            new (p.second) inner_node(id, prefix, val, branches));
   }

   inline inner_node::inner_node(object_id id, key_view prefix, object_id val, uint64_t branches)
       : _prefix_length(prefix.size()),
         _reserved_a(0),
         _reserved_b(0),
         _value(val),
         _present_bits(branches)
   {
      if constexpr (debug_nodes)
         std::cout << id.id << ": inner_node(): value=" << val.id << std::endl;
      memset(children(), 0, sizeof(object_id) * num_branches());
      memcpy(key_ptr(), prefix.data(), prefix.size());
   }
   /*
    *  Constructs a copy of in with the branches selected by 'branches'
    */
   inline inner_node::inner_node(object_id         id,
                                 ring_allocator&   a,
                                 const inner_node& in,
                                 key_view          prefix,
                                 object_id         val,
                                 uint64_t          branches)
       : _prefix_length(prefix.size()),
         _reserved_a(0),
         _reserved_b(0),
         _value(val),
         _present_bits(branches)
   {
      if constexpr (debug_nodes)
         std::cout << id.id << ": inner_node(): value=" << val.id << std::endl;
      if (in._present_bits == branches)
      {
         // memcpy( (char*)children(), (char*)in.children(), num_branches()*sizeof(object_id) );
         auto c  = children();
         auto ic = in.children();
         auto e  = c + num_branches();
         while (c != e)
         {
            if constexpr (debug_nodes)
               std::cout << id.id << ": inner_node(copy): bump child " << ic->id << std::endl;
            *c = bump_refcount_or_copy(a, *ic);
            ++c;
            ++ic;
         }
      }
      else
      {
         auto common_branches = in._present_bits & _present_bits;
         auto fb              = std::countr_zero(common_branches);
         while (fb < 64)
         {
            if constexpr (debug_nodes)
               std::cout << id.id << ": inner_node(copy): bump child " << in.branch(fb).id
                         << std::endl;
            branch(fb) = bump_refcount_or_copy(a, in.branch(fb));
            common_branches ^= 1ull << fb;
            fb = std::countr_zero(common_branches);
         }

         auto null_branches = (in._present_bits & branches) ^ branches;
         fb                 = std::countr_zero(null_branches);
         while (fb < 64)
         {
            branch(fb).id = 0;
            null_branches ^= 1ull << fb;
            fb = std::countr_zero(null_branches);
         }
      }
      memcpy(key_ptr(), prefix.data(), prefix.size());

      //assert(not is_value_node());
   }

   inline object_id& inner_node::branch(uint8_t b)
   {
      assert(branch_index(b) >= 0);
      if (branch_index(b) < 0) [[unlikely]]
         throw std::runtime_error("branch(b) <= 0, b: " + std::to_string(int(b)));
      return reinterpret_cast<object_id*>((char*)this + sizeof(inner_node))[branch_index(b)];
   }
   inline const object_id& inner_node::branch(uint8_t b) const
   {
      return const_cast<inner_node*>(this)->branch(b);
   }

   // @return num_children if not found
   inline int32_t inner_node::branch_index(uint32_t branch) const
   {
      assert(branch < 64);
      const uint32_t maskbits = branch % 64;
      const uint64_t mask     = -1ull >> (63 - maskbits);

      return std::popcount(_present_bits & mask) - 1;
   }

   // @return the first branch >= b
   inline uint8_t inner_node::lower_bound(uint8_t b) const
   {
      const uint64_t mask = (-1ull << (b & 63));
      return b >= 64 ? 64 : std::countr_zero(_present_bits & mask);
   }

   // last branch <= b
   inline int8_t inner_node::reverse_lower_bound(uint8_t b) const
   {
      const uint64_t mask = b == 63 ? -1ull : ~(-1ull << ((b + 1) & 63));
      return 63 - std::countl_zero(_present_bits & mask);
   }

   // @return the first branch > b, if b == 63 then
   // this will return 64
   inline uint8_t inner_node::upper_bound(uint8_t b) const
   {
      const uint64_t mask = (-1ull << ((b + 1) & 63));
      return b >= 63 ? 64 : std::countr_zero(_present_bits & mask);
   }

   inline void release_node(ring_allocator& ra, object_id obj)
   {
      if (!obj)
         return;
      auto [ptr, type] = ra.release(obj);
      if (ptr && type == node_type::inner)
      {
         auto& in = *reinterpret_cast<inner_node*>(ptr);
         if constexpr (debug_nodes)
            std::cout << obj.id << ": destroying; release value " << in.value().id << std::endl;
         release_node(ra, in.value());
         auto nb  = in.num_branches();
         auto pos = in.children();
         auto end = pos + nb;
         while (pos != end)
         {
            assert(*pos);
            if constexpr (debug_nodes)
               std::cout << obj.id << ": destroying; release child " << pos->id << std::endl;
            release_node(ra, *pos);
            ++pos;
         }
      }
      if (ptr && type == node_type::roots)
      {
         auto& vn    = *reinterpret_cast<value_node*>(ptr);
         auto  n     = vn.num_roots();
         auto  roots = vn.roots();
         while (n--)
         {
            if constexpr (debug_nodes)
               std::cout << obj.id << ": destroying; release root " << roots->id << std::endl;
            release_node(ra, *roots++);
         }
      }
   }

   inline location_lock copy_node(ring_allocator& ra, object_id id, char* ptr, node_type type)
   {
      if (type != node_type::inner)
      {
         auto src = reinterpret_cast<value_node*>(ptr);
         auto [lock, dest] =
             value_node::make(ra, src->key(), src->data(), type, type == node_type::roots);
         return std::move(lock);
      }
      else
      {
         auto src          = reinterpret_cast<inner_node*>(ptr);
         auto [lock, dest] = inner_node::make(
             ra, *src, src->key(), bump_refcount_or_copy(ra, src->value()), src->branches());
         return std::move(lock);
      }
   }

   inline object_id bump_refcount_or_copy(ring_allocator& ra, object_id id)
   {
      if (!id)
         return id;
      if constexpr (debug_nodes)
         std::cout << id.id << ": bump_refcount_or_copy" << std::endl;
      if (ra.bump_count(id))
         return id;
      auto [ptr, type, ref] = ra.get_cache<false>(id);
      return copy_node(ra, id, ptr, type).into_unlock_unchecked();
   }
}  // namespace triedent

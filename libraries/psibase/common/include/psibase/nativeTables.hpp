#pragma once

#include <psibase/block.hpp>
#include <psibase/db.hpp>

namespace psibase
{
   using NativeTableNum = uint16_t;

   static constexpr NativeTableNum statusTable         = 1;
   static constexpr NativeTableNum accountTable        = 2;
   static constexpr NativeTableNum codeByHashTable     = 3;
   static constexpr NativeTableNum databaseStatusTable = 4;

   static constexpr uint8_t nativeTablePrimaryIndex = 0;

   inline auto statusKey()
   {
      return std::tuple{statusTable, nativeTablePrimaryIndex};
   }
   struct StatusRow
   {
      Checksum256              chainId;
      BlockHeader              current;
      std::optional<BlockInfo> head;
      uint32_t                 numExecutionMemories = 32;  // TODO: move to a configuration table

      static constexpr auto db = psibase::DbId::nativeUnconstrained;
      static auto           key() { return statusKey(); }
   };
   PSIO_REFLECT(StatusRow, chainId, current, head, numExecutionMemories)

   // TODO: Rename account to contract?
   inline auto accountKey(AccountNumber num)
   {
      // TODO: leave space for secondary index?
      return std::tuple{accountTable, nativeTablePrimaryIndex, num};
   }
   struct AccountRow
   {
      // Constants for flags
      static constexpr uint64_t allowSudo            = uint64_t(1) << 0;
      static constexpr uint64_t allowWriteNative     = uint64_t(1) << 1;
      static constexpr uint64_t isSubjective         = uint64_t(1) << 2;
      static constexpr uint64_t allowWriteSubjective = uint64_t(1) << 3;
      static constexpr uint64_t canNotTimeOut        = uint64_t(1) << 4;
      static constexpr uint64_t canSetTimeLimit      = uint64_t(1) << 5;

      AccountNumber num;           // TODO: rename
      AccountNumber authContract;  // TODO: move out of native
      uint64_t      flags = 0;     // Constants above

      // TODO?  1 account with contract per 1000+ without - faster perf on dispatch because don't need to lookup new account
      Checksum256 codeHash  = {};
      uint8_t     vmType    = 0;
      uint8_t     vmVersion = 0;
      //==================================^^

      static constexpr auto db = psibase::DbId::nativeUnconstrained;
      auto                  key() const { return accountKey(num); }
   };
   PSIO_REFLECT(AccountRow, num, authContract, flags, codeHash, vmType, vmVersion)

   inline auto codeByHashKey(const Checksum256& codeHash, uint8_t vmType, uint8_t vmVersion)
   {
      // TODO: leave space for secondary index?
      return std::tuple{codeByHashTable, nativeTablePrimaryIndex, codeHash, vmType, vmVersion};
   }

   /// where code is actually stored, duplicate contracts are reused
   struct CodeByHashRow
   {
      Checksum256 codeHash  = {};
      uint8_t     vmType    = 0;
      uint8_t     vmVersion = 0;

      uint32_t             numRefs = 0;   // number accounts that ref this
      std::vector<uint8_t> code    = {};  // actual code, TODO: compressed

      // The code table is in nativeConstrained. The native code
      // verifies codeHash and the key. This prevents a poison block
      // that could happen if the key->code map doesn't match the
      // key->(jitted code) map or the key->(optimized code) map.
      static constexpr auto db = psibase::DbId::nativeConstrained;
      auto                  key() const { return codeByHashKey(codeHash, vmType, vmVersion); }
   };
   PSIO_REFLECT(CodeByHashRow, codeHash, vmType, vmVersion, numRefs, code)

   inline auto databaseStatusKey()
   {
      return std::tuple{databaseStatusTable, nativeTablePrimaryIndex};
   }
   struct DatabaseStatusRow
   {
      uint64_t nextHistoryEventNumber = 1;
      uint64_t nextUIEventNumber      = 1;

      // This table is in nativeConstrained. The native code blocks contracts
      // from writing to this since it could break backing stores.
      static constexpr auto db = psibase::DbId::nativeConstrained;
      static auto           key() { return databaseStatusKey(); }
   };
   PSIO_REFLECT(DatabaseStatusRow, nextHistoryEventNumber, nextUIEventNumber)

}  // namespace psibase

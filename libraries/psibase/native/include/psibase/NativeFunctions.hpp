#pragma once

#include <psibase/ExecutionContext.hpp>

#include <eosio/vm/span.hpp>
#include <psibase/nativeTables.hpp>

namespace psibase
{
   struct AccountRow;

   struct NativeFunctions
   {
      Database&           database;
      TransactionContext& transactionContext;
      bool                isReadOnly        = false;
      AccountRow          contractAccount   = {};
      ActionContext*      currentActContext = nullptr;  // Changes during recursion

      std::vector<char> result_key;
      std::vector<char> result_value;

      // TODO: rename kvPutSequential, kvGetSequential
      // TODO: clean up which functions do/don't clear result_*
      // TODO: delete range. Need some way for system contracts to enable/disable it
      //       since it's only compatible with some resource models
      // TODO: some way for transaction-sys to indicate auth failures.
      //       Maybe not an intrinsic? Is there a way to tie this into the
      //       subjective mechanics?
      // TODO: related to ^. Some way to bill failed transactions after the first
      //       authorizer has been verified. But... custom proof and auth contracts
      //       could abuse that. Maybe a time limit for the proofs and the first auth
      //       contract? Can't limit just a single proof since first auth could depend
      //       on several proofs.
      // TODO: related to ^. See TODOs on NativeFunctions::call

      uint32_t getResult(eosio::vm::span<char> dest, uint32_t offset);
      uint32_t getKey(eosio::vm::span<char> dest);
      void     writeConsole(eosio::vm::span<const char> str);
      void     abortMessage(eosio::vm::span<const char> str);
      uint64_t getBillableTime();
      void     setMaxTransactionTime(uint64_t nanoseconds);
      uint32_t getCurrentAction();
      uint32_t call(eosio::vm::span<const char> data);
      void     setRetval(eosio::vm::span<const char> data);
      void kvPut(uint32_t db, eosio::vm::span<const char> key, eosio::vm::span<const char> value);
      uint64_t kvPutSequential(uint32_t db, eosio::vm::span<const char> value);
      void     kvRemove(uint32_t db, eosio::vm::span<const char> key);
      uint32_t kvGet(uint32_t db, eosio::vm::span<const char> key);
      uint32_t kvGetSequential(uint32_t db, uint64_t indexNumber);
      uint32_t kvGreaterEqual(uint32_t db, eosio::vm::span<const char> key, uint32_t matchKeySize);
      uint32_t kvLessThan(uint32_t db, eosio::vm::span<const char> key, uint32_t matchKeySize);
      uint32_t kvMax(uint32_t db, eosio::vm::span<const char> key);
      uint32_t kvGetTransactionUsage();
   };  // NativeFunctions
}  // namespace psibase

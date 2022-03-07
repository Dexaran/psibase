#pragma once

#include <psibase/intrinsic.hpp>
#include <psibase/native_tables.hpp>

namespace auth_fake_sys
{
   static constexpr psibase::account_num contract = 4;

   struct auth_check
   {
      psibase::action             action;
      std::vector<psibase::claim> claims;
   };
   EOSIO_REFLECT(auth_check, action, claims)

   using action = std::variant<auth_check>;

   template <typename T, typename R = typename T::return_type>
   R call(psibase::account_num sender, T args)
   {
      auto result = psibase::call(psibase::action{
          .sender   = sender,
          .contract = contract,
          .raw_data = eosio::convert_to_bin(action{std::move(args)}),
      });
      if constexpr (!std::is_same_v<R, void>)
         return eosio::convert_from_bin<R>(result);
   }
}  // namespace auth_fake_sys

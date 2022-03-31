#pragma once
#include <psibase/intrinsic.hpp>
#include <psio/fracpack.hpp>

namespace psibase
{

   /*
   namespace detail
   {
      struct FRACPACK raw_variant_view
      {
         uint8_t  action_idx;
         uint32_t size;
         char     data[];
      };

   };  // namespace detail
   */

   template <typename R, typename T, typename MemberPtr, typename... Args>
   void call_method(T& contract, MemberPtr method, Args&&... args)
   {
      psio::shared_view_ptr<R> p((contract.*method)(std::forward<decltype(args)>(args)...));
      raw::set_retval(p.data(), p.size());
   }

   /**
    *  This method is called when a contract receives a call and will
    *  and will call the proper method on Contact assuming Contract has
    *  used the PSIO_REFLECT_INTERFACE macro.
    */
   template <typename Contract>
   void dispatch(account_num sender, account_num receiver)
   {
      Contract contract;
      contract.psibase::contract::dispatch_set_sender_receiver(sender, receiver);
      action act = get_current_action();  /// action view...

      bool called = psio::reflect<Contract>::get_by_name(
          act.method.value,
          [&](auto member_func)
          {
             using result_type = decltype(psio::result_of(member_func));
             using param_tuple =
                 decltype(psio::tuple_remove_view(psio::args_as_tuple(member_func)));

             psibase::check(psio::fracvalidate<param_tuple>(
                                act.raw_data.data(), act.raw_data.data() + act.raw_data.size())
                                .valid,
                            "invalid argument encoding");
             psio::const_view<param_tuple> param_view(act.raw_data.data());

             param_view->call(
                 [&](auto... args)
                 {
                    if constexpr (std::is_same_v<void, result_type>)
                    {
                       (contract.*member_func)(std::forward<decltype(args)>(args)...);
                    }
                    else
                    {
                       call_method<result_type, Contract, decltype(member_func), decltype(args)...>(
                           contract, member_func, std::forward<decltype(args)>(args)...);
                    }
                 });  // param_view::call
          });         // reflect::get
      check(called, "unknown contract action");
   }  // dispatch

}  // namespace psibase

#define PSIBASE_DISPATCH(CONTRACT)                                                    \
   extern "C" void called(psibase::account_num receiver, psibase::account_num sender) \
   {                                                                                  \
      psibase::dispatch<CONTRACT>(sender, receiver);                                  \
   }                                                                                  \
   extern "C" void __wasm_call_ctors();                                               \
   extern "C" void start(psibase::account_num this_contract) { __wasm_call_ctors(); } \
   \

#include <contracts/system/AuthFakeSys.hpp>

#include <psibase/dispatch.hpp>
#include <psibase/print.hpp>

static constexpr bool enable_print = false;

namespace system_contract
{
   void AuthFakeSys::checkAuthSys(psibase::Action             action,
                                  std::vector<psibase::Claim> claims,
                                  bool                        firstAuth,
                                  bool                        readOnly)
   {
      if (enable_print)
         psibase::print("auth_check\n");
   }
}  // namespace system_contract

PSIBASE_DISPATCH(system_contract::AuthFakeSys)

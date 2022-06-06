#include "contracts/system/CommonSys.hpp"

#include <contracts/system/ProxySys.hpp>
#include <psibase/dispatch.hpp>
#include <psibase/nativeTables.hpp>
#include <psibase/serveContent.hpp>
#include <psio/to_json.hpp>

static constexpr bool enable_print = false;

using namespace psibase;
using Tables = psibase::ContractTables<psibase::WebContentTable>;

namespace psibase
{
   std::optional<RpcReplyData> CommonSys::serveSys(RpcRequestData request)
   {
      auto to_json = [](const auto& obj)
      {
         auto json = psio::convert_to_json(obj);
         return RpcReplyData{
             .contentType = "application/json",
             .body        = {json.begin(), json.end()},
         };
      };

      if (request.method == "GET")
      {
         if (request.target == "/common/thiscontract")
         {
            std::string contractName;
            if (request.host.size() > request.rootHost.size() + 1 &&
                request.host.ends_with(request.rootHost) &&
                request.host[request.host.size() - request.rootHost.size() - 1] == '.')
               contractName.assign(request.host.begin(),
                                   request.host.end() - request.rootHost.size() - 1);
            else
               contractName = "common-sys";
            return to_json(contractName);
         }
         if (request.target == "/common/rootdomain")
            return to_json(request.rootHost);
         if (request.target == "/common/rootdomain.js")
         {
            auto js = "const rootdomain = '" + request.rootHost + "';\n";
            js +=
                "function siblingUrl(baseUrl, contract, path) {\n"
                "    let loc;\n"
                "    if (!baseUrl)\n"
                "        loc = location;\n"
                "    else\n"
                "        loc = new URL(baseUrl);\n"
                "    return loc.protocol + '//' + (contract ? contract + '.' : '') + rootdomain +\n"
                "           ':' + loc.port + '/' + (path || '').replace(/^\\/+/, '');\n"
                "}\n";
            return RpcReplyData{
                .contentType = "text/javascript",
                .body        = {js.begin(), js.end()},
            };
         }
         if (request.target == "/common/rootdomain.mjs")
         {
            auto js = "export const rootdomain = '" + request.rootHost + "';\n";
            js +=
                "export function siblingUrl(baseUrl, contract, path) {\n"
                "    let loc;\n"
                "    if (!baseUrl)\n"
                "        loc = location;\n"
                "    else\n"
                "        loc = new URL(baseUrl);\n"
                "    return loc.protocol + '//' + (contract ? contract + '.' : '') + rootdomain +\n"
                "           ':' + loc.port + '/' + (path || '').replace(/^\\/+/, '');\n"
                "}\n";
            return RpcReplyData{
                .contentType = "text/javascript",
                .body        = {js.begin(), js.end()},
            };
         }
      }

      if (request.method == "POST")
      {
         if (request.target == "/common/pack/Transaction")
         {
            request.body.push_back(0);
            psio::json_token_stream jstream{request.body.data()};
            Transaction             trx;
            psio::from_json(trx, jstream);
            return RpcReplyData{
                .contentType = "application/octet-stream",
                .body        = psio::convert_to_frac(trx),
            };
         }
         if (request.target == "/common/pack/SignedTransaction")
         {
            request.body.push_back(0);
            psio::json_token_stream jstream{request.body.data()};
            SignedTransaction       trx;
            psio::from_json(trx, jstream);
            return RpcReplyData{
                .contentType = "application/octet-stream",
                .body        = psio::convert_to_frac(trx),
            };
         }
      }

      if (auto result = psibase::serveContent(request, Tables{getReceiver()}))
         return result;
      return std::nullopt;
   }  // CommonSys::serveSys

   void CommonSys::storeSys(std::string path, std::string contentType, std::vector<char> content)
   {
      psibase::check(getSender() == getReceiver(), "wrong sender");
      psibase::storeContent(std::move(path), std::move(contentType), std::move(content),
                            Tables{getReceiver()});
   }

}  // namespace psibase

PSIBASE_DISPATCH(psibase::CommonSys)

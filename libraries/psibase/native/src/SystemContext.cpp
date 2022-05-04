#include <psibase/ActionContext.hpp>

#include <mutex>

namespace psibase
{
   struct shared_state_impl
   {
      std::mutex                                   mutex;
      SharedDatabase                               db;
      psibase::WasmCache                           wasmCache;
      std::vector<std::unique_ptr<system_context>> system_context_cache;

      shared_state_impl(SharedDatabase db, psibase::WasmCache wasmCache)
          : db{std::move(db)}, wasmCache{std::move(wasmCache)}
      {
      }
   };

   shared_state::shared_state(SharedDatabase db, psibase::WasmCache wasmCache)
       : impl{std::make_unique<shared_state_impl>(std::move(db), std::move(wasmCache))}
   {
   }

   shared_state::~shared_state() {}

   std::unique_ptr<system_context> shared_state::get_system_context()
   {
      std::lock_guard<std::mutex> lock{impl->mutex};
      if (impl->system_context_cache.empty())
         return std::make_unique<system_context>(system_context{impl->db, impl->wasmCache});
      auto result = std::move(impl->system_context_cache.back());
      impl->system_context_cache.pop_back();
      return result;
   }

   void shared_state::add_system_context(std::unique_ptr<system_context> context)
   {
      std::lock_guard<std::mutex> lock{impl->mutex};
      impl->system_context_cache.push_back(std::move(context));
   }
}  // namespace psibase

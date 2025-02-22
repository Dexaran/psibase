cmake_minimum_required(VERSION 3.16.3)
project(psibase)
include(ExternalProject)

if(CMAKE_BINARY_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR)
    message(FATAL_ERROR "Building in-source is not supported; create a build dir and remove ${CMAKE_SOURCE_DIR}/CMakeCache.txt and ${CMAKE_SOURCE_DIR}/CMakeFiles")
endif()

if(NOT DEFINED WASI_SDK_PREFIX AND DEFINED ENV{WASI_SDK_PREFIX})
    set(WASI_SDK_PREFIX $ENV{WASI_SDK_PREFIX})
endif()


option (FORCE_COLORED_OUTPUT "Always produce ANSI-colored output (GNU/Clang only)." TRUE)
if (${FORCE_COLORED_OUTPUT})
    if ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "GNU")
       add_compile_options (-fdiagnostics-color=always)
    else ()
       add_compile_options(-fcolor-diagnostics)
    endif ()
endif ()

file(WRITE ${CMAKE_BINARY_DIR}/CTestTestfile.cmake)

option(CLIO_ONLY "Only build clio" OFF)

option(BUILD_NATIVE "Build native code" ON)
if(BUILD_NATIVE)
    add_subdirectory(native)
    file(APPEND ${CMAKE_BINARY_DIR}/CTestTestfile.cmake "subdirs(\"native\")\n")
endif()

option(BUILD_RELEASE_WASM "Build release wasms" ON)
option(BUILD_DEBUG_WASM "Build debug wasms" OFF)

if(DEFINED WASI_SDK_PREFIX AND NOT CLIO_ONLY)
    ExternalProject_Add(wasm
        SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/wasm
        BINARY_DIR wasm
        INSTALL_COMMAND ""
        BUILD_ALWAYS 1
        TEST_EXCLUDE_FROM_MAIN 1
        STEP_TARGETS configure
        CMAKE_ARGS
            -DCMAKE_BUILD_TYPE=
            -DBUILD_RELEASE_WASM=${BUILD_RELEASE_WASM}
            -DBUILD_DEBUG_WASM=${BUILD_DEBUG_WASM}
            -DCMAKE_TARGET_MESSAGES=${CMAKE_TARGET_MESSAGES}
            -DCMAKE_TOOLCHAIN_FILE=${CMAKE_CURRENT_SOURCE_DIR}/wasm/toolchain.cmake
            -DWASI_SDK_PREFIX=${WASI_SDK_PREFIX}
            -DWASM_CLANG_PREFIX=${WASM_CLANG_PREFIX}
            -DWASM_CLANG_SUFFIX=${WASM_CLANG_SUFFIX}
            -DCMAKE_SYSROOT=${WASI_SDK_PREFIX}/share/wasi-sysroot
            -DFORCE_COLORED_OUTPUT=${FORCE_COLORED_OUTPUT}
            -DCMAKE_C_COMPILER_LAUNCHER=${CMAKE_C_COMPILER_LAUNCHER}
            -DCMAKE_CXX_COMPILER_LAUNCHER=${CMAKE_CXX_COMPILER_LAUNCHER}
            -DCMAKE_FIND_ROOT_PATH=${CMAKE_CURRENT_BINARY_DIR}/wasm/deps
            -DCMAKE_PREFIX_PATH=/
    )
    file(APPEND ${CMAKE_BINARY_DIR}/CTestTestfile.cmake "subdirs(\"wasm\")\n")
    ExternalProject_Add_StepTargets(wasm test)
else()
    message(WARNING "WASI_SDK_PREFIX isn't defined; skipping wasm")
endif()

option(BUILD_DOC "Build doc" OFF)
if(BUILD_DOC)
    set(DOC_DEP doc)
    ExternalProject_Add(
        gen-cpp-doc
        DEPENDS             wasm
        SOURCE_DIR          ${CMAKE_CURRENT_SOURCE_DIR}/rust
        CONFIGURE_COMMAND   ""
        BUILD_COMMAND       cargo build -r --bin gen-cpp-doc --manifest-path ${CMAKE_CURRENT_SOURCE_DIR}/rust/Cargo.toml --target-dir ${CMAKE_CURRENT_BINARY_DIR}/rust
        INSTALL_COMMAND     ""
    )
    configure_file(doc/psidk/book.toml.in doc/psidk/book.toml)
    configure_file(doc/psidk/mermaid.min.js doc/psidk/mermaid.min.js COPYONLY)
    configure_file(doc/psidk/mermaid-init.js doc/psidk/mermaid-init.js COPYONLY)
    ExternalProject_Add(
        ${DOC_DEP}
        DEPENDS             gen-cpp-doc
        SOURCE_DIR          ${CMAKE_CURRENT_SOURCE_DIR}/doc/psidk
        CONFIGURE_COMMAND   ""
        BUILD_COMMAND       export WASI_SDK_PREFIX=${WASI_SDK_PREFIX} && cd ${CMAKE_CURRENT_SOURCE_DIR}/doc/psidk && rm -rf ${CMAKE_CURRENT_BINARY_DIR}/book && mdbook build -d ${CMAKE_CURRENT_BINARY_DIR}/book ${CMAKE_CURRENT_BINARY_DIR}/doc/psidk
        INSTALL_COMMAND     ""
    )
endif()

set(JS_DIRS
    services/system/AccountSys/ui:AccountSys_js
    services/user/PsiSpaceSys/ui:PsiSpaceSys_js
    services/user/CommonSys/ui:CommonSys_js
    services/user/TokenSys/ui:TokenSys_js
    services/user/ExploreSys/ui:ExploreSys_js
    services/user/AdminSys/ui:AdminSys_js
)

option(BUILD_JS "Build JS" ON)
if(BUILD_JS)
    list(TRANSFORM JS_DIRS REPLACE "^[^:]+:([^:]+)$" \\1 OUTPUT_VARIABLE JS_DEPS)
    foreach(ENTRY ${JS_DIRS})
        string(REGEX REPLACE "^([^:]+):([^:]+)$" \\1 PATH ${ENTRY})
        string(REGEX REPLACE "^([^:]+):([^:]+)$" \\2 DEP ${ENTRY})
        add_custom_target(
            ${DEP}
            COMMAND cd ${CMAKE_CURRENT_SOURCE_DIR}/${PATH} && yarn && yarn build
        )
    endforeach()
endif()

if(BUILD_JS)
    function(psinode_files target pfx)
    add_custom_target(
    ${target}
    ALL
    DEPENDS AdminSys_js
    COMMAND rm -rf ${pfx}/share/psibase/services/admin-sys
    COMMAND mkdir -p ${pfx}/share/psibase/services/admin-sys/common
    COMMAND cp -a -T ${CMAKE_CURRENT_SOURCE_DIR}/services/user/AdminSys/ui/dist ${pfx}/share/psibase/services/admin-sys
    COMMAND cp -a -T ${CMAKE_CURRENT_SOURCE_DIR}/services/user/CommonSys/common/fonts ${pfx}/share/psibase/services/admin-sys/common/fonts
    COMMAND cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/user/CommonSys/common/rpc.mjs ${pfx}/share/psibase/services/admin-sys/common/
    COMMAND cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/user/CommonSys/common/keyConversions.mjs ${pfx}/share/psibase/services/admin-sys/common/
    COMMAND cp ${CMAKE_CURRENT_SOURCE_DIR}/programs/psinode/config.in ${pfx}/share/psibase/config.in
    )
    endfunction()

    psinode_files(psinode-share ${CMAKE_CURRENT_BINARY_DIR})
    psinode_files(psinode-psidk-share ${CMAKE_CURRENT_BINARY_DIR}/psidk)
endif()

option(BUILD_RUST "Build rust" ON)
if(BUILD_RUST)
    set(BOOT_IMAGE ${CMAKE_CURRENT_SOURCE_DIR}/rust/psibase/boot-image)
    set(COMMON_SYS ${CMAKE_CURRENT_SOURCE_DIR}/services/user/CommonSys)
    set(THIRD_SRC ${COMMON_SYS}/common/thirdParty/src)
    set(THIRD_DEST ${BOOT_IMAGE}/CommonSys/common/thirdParty)
    ExternalProject_Add(
        rust
        DEPENDS wasm ${DOC_DEP} ${JS_DEPS}
        SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust
        CONFIGURE_COMMAND ""
        BUILD_COMMAND   rm -rf ${BOOT_IMAGE}
        COMMAND         mkdir -p ${BOOT_IMAGE}/CommonSys/common ${BOOT_IMAGE}/CommonSys/common/fonts ${BOOT_IMAGE}/CommonSys/ui/vanilla
        COMMAND         mkdir -p ${BOOT_IMAGE}/AccountSys/ui ${BOOT_IMAGE}/AdminSys/ui ${BOOT_IMAGE}/ExploreSys/ui
        COMMAND         mkdir -p ${BOOT_IMAGE}/AuthEcSys ${BOOT_IMAGE}/TokenSys/ui ${BOOT_IMAGE}/PsiSpaceSys/ui
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/AccountSys.wasm                   ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/AdminSys.wasm                     ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/AuthEcSys.wasm                    ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/AuthAnySys.wasm                   ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/CommonSys.wasm                    ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/ExploreSys.wasm                   ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/NftSys.wasm                       ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/ProducerSys.wasm                  ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/ProxySys.wasm                     ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/PsiSpaceSys.wasm                  ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/RAccountSys.wasm                  ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/RAuthEcSys.wasm                   ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/RProducerSys.wasm                 ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/RProxySys.wasm                    ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/RTokenSys.wasm                    ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/SetCodeSys.wasm                   ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/SymbolSys.wasm                    ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/TokenSys.wasm                     ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/TransactionSys.wasm               ${BOOT_IMAGE}
        COMMAND         cp -a ${CMAKE_CURRENT_BINARY_DIR}/VerifyEcSys.wasm                  ${BOOT_IMAGE}

        COMMAND         cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/system/AccountSys/ui/dist    ${BOOT_IMAGE}/AccountSys/ui/dist
        COMMAND         bash -c "cp -a ${COMMON_SYS}/common/*.mjs                               ${BOOT_IMAGE}/CommonSys/common/"
        COMMAND         bash -c "cp -a ${COMMON_SYS}/common/fonts/*                             ${BOOT_IMAGE}/CommonSys/common/fonts/"
        COMMAND         cp -a ${COMMON_SYS}/ui/vanilla/common.index.html                        ${BOOT_IMAGE}/CommonSys/ui/vanilla
        COMMAND         cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/user/CommonSys/ui/dist       ${BOOT_IMAGE}/CommonSys/ui/dist
        COMMAND         cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/user/ExploreSys/ui/dist      ${BOOT_IMAGE}/ExploreSys/ui/dist
        COMMAND         cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/user/TokenSys/ui/dist        ${BOOT_IMAGE}/TokenSys/ui/dist
        COMMAND         cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/user/PsiSpaceSys/ui/dist     ${BOOT_IMAGE}/PsiSpaceSys/ui/dist
        COMMAND         cp -a ${CMAKE_CURRENT_SOURCE_DIR}/services/user/AdminSys/ui/dist        ${BOOT_IMAGE}/AdminSys/ui/dist

        COMMAND         bash -c "cp -a ${CMAKE_CURRENT_BINARY_DIR}/book                     ${BOOT_IMAGE}/doc || mkdir -p ${BOOT_IMAGE}/doc"

        COMMAND         rm -rf ${THIRD_DEST}
        COMMAND         mkdir -p ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/htm.module.js                                    ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/iframeResizer.contentWindow.js                   ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/iframeResizer.js                                 ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/react-dom.development.js                         ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/react-dom.production.min.js                      ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/react-router-dom.min.js                          ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/react.development.js                             ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/react.production.min.js                          ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/semantic-ui-react.min.js                         ${THIRD_DEST}
        COMMAND         cp -a ${THIRD_SRC}/useLocalStorageState.js                          ${THIRD_DEST}

        COMMAND         touch ${CMAKE_CURRENT_SOURCE_DIR}/rust/psibase/src/boot.rs
        COMMAND         cargo build -r --bin psibase --manifest-path ${CMAKE_CURRENT_SOURCE_DIR}/rust/Cargo.toml --target-dir ${CMAKE_CURRENT_BINARY_DIR}/rust

        INSTALL_COMMAND mkdir -p ${CMAKE_CURRENT_BINARY_DIR}/psidk/bin
        COMMAND cp ${CMAKE_CURRENT_BINARY_DIR}/rust/release/psibase ${CMAKE_CURRENT_BINARY_DIR}/psidk/bin
    )
endif()

option(BUILD_EXAMPLES "Build examples" ON)
if(BUILD_EXAMPLES)
    add_subdirectory(doc/psidk/src/cpp-service doc-examples)
endif()

set(CMAKE_EXPORT_COMPILE_COMMANDS on)

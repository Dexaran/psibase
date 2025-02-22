cmake_minimum_required(VERSION 3.16.3)
cmake_policy(VERSION 3.16)
project(wasm)
enable_testing()

set(IS_WASM YES)
get_filename_component(ROOT_SOURCE_DIR ${CMAKE_SOURCE_DIR}/.. ABSOLUTE)
get_filename_component(ROOT_BINARY_DIR ${CMAKE_BINARY_DIR}/.. ABSOLUTE)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(FORCE_COLORED_OUTPUT "Always produce ANSI-colored output" TRUE)
if(${FORCE_COLORED_OUTPUT})
    add_compile_options(-fcolor-diagnostics)
endif()

option(BUILD_RELEASE_WASM "Build release wasms" ON)
option(BUILD_DEBUG_WASM "Build debug wasms" OFF)

# Point to psidk-config.cmake to allow documentation examples to build
set(psidk_DIR ${CMAKE_SOURCE_DIR})

function(native_test N)
endfunction()

function(add_wasm_test N)
    add_test(
        NAME t-${N}
        WORKING_DIRECTORY ${ROOT_BINARY_DIR}
        COMMAND ./psitest -v ${N}.wasm -s
    )
    set_tests_properties(t-${N} PROPERTIES ENVIRONMENT NODE_PATH=dist)
endfunction()

function(add_wasm_test_release N)
    if(BUILD_RELEASE_WASM)
        add_wasm_test(${N})
    endif()
endfunction()

function(native_and_wasm_test N)
    add_test(
        NAME t-${N}
        WORKING_DIRECTORY ${ROOT_BINARY_DIR}
        COMMAND ./psitest  ${N}.wasm -s
    )
    set_tests_properties(t-${N} PROPERTIES ENVIRONMENT NODE_PATH=dist)
endfunction()

function(copy_headers from to)
    file(GLOB foo
        ${CMAKE_CURRENT_SOURCE_DIR}/${from}/*.h
        ${CMAKE_CURRENT_SOURCE_DIR}/${from}/*.hpp
        ${CMAKE_CURRENT_SOURCE_DIR}/${from}/*.hxx
    )
    file(COPY ${foo} DESTINATION ${ROOT_BINARY_DIR}/psidk/${to})
endfunction()

macro(conditional_add)
    if(BUILD_RELEASE_WASM)
        add("")
    endif()
    if(BUILD_DEBUG_WASM)
        add("-debug")
    endif()
endmacro()

file(COPY ${ROOT_SOURCE_DIR}/LICENSE DESTINATION ${ROOT_BINARY_DIR}/psidk/licenses)

add_library(wasm-base INTERFACE)
target_compile_options(wasm-base INTERFACE -fno-exceptions -DCOMPILING_WASM -mthread-model single -O3)
target_link_options(wasm-base INTERFACE -Wl,--strip-all -O3 -L${ROOT_BINARY_DIR}/wasm/deps/lib)

add_library(wasm-base-debug INTERFACE)
target_compile_options(wasm-base-debug INTERFACE -fno-exceptions -DCOMPILING_WASM -mthread-model single -ggdb)
target_link_options(wasm-base-debug INTERFACE -ggdb -L${ROOT_BINARY_DIR}/wasm/deps/lib)

# -psidk variant of build targets need paths converted
function(add_psidk_base_lib suffix)
    add_library(wasm-base${suffix}-psidk INTERFACE)
    target_link_libraries(wasm-base${suffix}-psidk INTERFACE wasm-base${suffix})
    target_compile_options(wasm-base${suffix}-psidk INTERFACE
            -ffile-prefix-map=${ROOT_BINARY_DIR}/wasm/boost/=psidk:/boost/
            -ffile-prefix-map=${ROOT_BINARY_DIR}/wasm/catch2/=psidk:/catch2/
            -ffile-prefix-map=${ROOT_SOURCE_DIR}/services/=psidk:/services/
            -ffile-prefix-map=${ROOT_SOURCE_DIR}/external/=psidk:/
            -ffile-prefix-map=${ROOT_SOURCE_DIR}/libraries/=psidk:/
            -ffile-prefix-map=${WASI_SDK_PREFIX}/=psidk-wasi-sdk:/
    )
endfunction()
add_psidk_base_lib("")
add_psidk_base_lib("-debug")

set(DEP_URL https://github.com/eoscommunity/Eden/releases/download/deps)
set(GMP_ARCHIVE gmp-6.2.1.tar.zst)
set(GMP_EXTRACTED gmp-6.2.1)
set(OPENSSL_ARCHIVE openssl-1.1.1k.tar.gz)
set(OPENSSL_EXTRACTED openssl-1.1.1k)
set(ZLIB_ARCHIVE zlib-1.2.11.tar.gz)
set(ZLIB_EXTRACTED zlib-1.2.11)
set(FLAT_BUFFERS_URL https://github.com/google/flatbuffers/archive/refs/tags )
set(FLAT_BUFFERS_ARCHIVE v2.0.0.tar.gz )
set(FLAT_BUFFERS_EXTRACTED flatbuffers )
set(DEP_PREFIX ${CMAKE_BINARY_DIR}/deps)

function(download url archive)
    if(NOT EXISTS ${archive})
        message("Downloading ${url}")
        file(DOWNLOAD ${url} ${archive}
            STATUS download_status
            TIMEOUT 600
            SHOW_PROGRESS
            TLS_VERIFY ON)
        list(POP_BACK download_status BOOST_DOWNLOAD_STATUS_MSG)
        if(NOT download_status EQUAL 0)
            file(REMOVE ${archive})
            message(FATAL_ERROR "Download ${url} failed. ${BOOST_DOWNLOAD_STATUS_MSG}")
        endif()
    endif()
endfunction()

# These builds are done at generation time so cmake's find_* commands can find them
if(NOT EXISTS ${DEP_PREFIX}/lib/libz.a)
    download(${DEP_URL}/${ZLIB_ARCHIVE} ${DEP_PREFIX}/${ZLIB_ARCHIVE})
    message("building zlib")
    execute_process(
        WORKING_DIRECTORY ${DEP_PREFIX}
        COMMAND bash -c "\
               rm -rf ${ZLIB_EXTRACTED}                                             \
            && tar xf ${ZLIB_ARCHIVE}                                               \
            && cd ${ZLIB_EXTRACTED}                                                 \
            && export CC=\"${CMAKE_C_COMPILER_LAUNCHER} ${CMAKE_C_COMPILER}\"       \
            && export CXX=\"${CMAKE_CXX_COMPILER_LAUNCHER} ${CMAKE_CXX_COMPILER}\"  \
            && export AR=${CMAKE_AR}                                                \
            && export RANLIB=${CMAKE_RANLIB}                                        \
            && export CFLAGS=\"--sysroot=${CMAKE_SYSROOT}\"                         \
            && ./configure --prefix=${DEP_PREFIX} --static                          \
            && make -j                                                              \
            && make -j install                                                      \
        "
    )
endif()

if(NOT EXISTS ${DEP_PREFIX}/lib/libgmp.a)
    download(${DEP_URL}/${GMP_ARCHIVE} ${DEP_PREFIX}/${GMP_ARCHIVE})
    message("building gmp")
    execute_process(
        WORKING_DIRECTORY ${DEP_PREFIX}
        COMMAND bash -c "\
               rm -rf ${GMP_EXTRACTED}                                                      \
            && tar xf ${GMP_ARCHIVE} -I zstd                                                \
            && cd ${GMP_EXTRACTED}                                                          \
            && ./configure                                                                  \
                CC=\"${CMAKE_C_COMPILER_LAUNCHER} ${CMAKE_C_COMPILER}\"                     \
                CFLAGS=\"--sysroot=${CMAKE_SYSROOT} -D_WASI_EMULATED_SIGNAL\"               \
                LDFLAGS=-lwasi-emulated-signal                                              \
                --prefix=${DEP_PREFIX}                                                      \
                --host none                                                                 \
                --disable-assembly   \                                                      \
            && make -j                                                                      \
            && make -j install                                                              \
        "
    )
endif()

if(NOT EXISTS ${DEP_PREFIX}/lib/libssl.a)
    download(${DEP_URL}/${OPENSSL_ARCHIVE} ${DEP_PREFIX}/${OPENSSL_ARCHIVE})
    message("building openssl")
    set(OPENSSL_BUILD_FLAGS "--sysroot=${CMAKE_SYSROOT} -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_PROCESS_CLOCKS -DNO_SYSLOG -DOPENSSL_NO_SECURE_MEMORY")
    execute_process(
        WORKING_DIRECTORY ${DEP_PREFIX}
        COMMAND bash -c "\
               rm -rf ${OPENSSL_EXTRACTED}                                                  \
            && tar xf ${OPENSSL_ARCHIVE}                                                    \
            && cd ${OPENSSL_EXTRACTED}                                                      \
            && ./Configure linux-generic32                                                  \
                --prefix=${DEP_PREFIX}                                                      \
                no-afalgeng no-asm no-dso no-shared no-sock no-tests no-threads no-ui       \
                CC=\"${CMAKE_C_COMPILER_LAUNCHER} ${CMAKE_C_COMPILER}\"                     \
                CXX=\"${CMAKE_CXX_COMPILER_LAUNCHER} ${CMAKE_CXX_COMPILER}\"                \
                AR=${CMAKE_AR}                                                              \
                RANLIB=${CMAKE_RANLIB}                                                      \
                CFLAGS=\"${OPENSSL_BUILD_FLAGS}\"                                           \
                CXXFLAGS=\"${OPENSSL_BUILD_FLAGS}\"                                         \
            && make -j install_dev install_engines                                          \
        "
    )
endif()

if(NOT EXISTS ${DEP_PREFIX}/lib/libsecp256k1.a)
    message("building secp256k1")
    execute_process(
        WORKING_DIRECTORY ${DEP_PREFIX}
        COMMAND bash -c "\
               rm -rf secp256k1                                                         \
            && cp -r ${ROOT_SOURCE_DIR}/external/secp256k1 secp256k1                    \
            && cd secp256k1                                                             \
            && ./autogen.sh                                                             \
            && export CC=\"${CMAKE_C_COMPILER_LAUNCHER} ${CMAKE_C_COMPILER}\"           \
            && export AR=${CMAKE_AR}                                                    \
            && export RANLIB=${CMAKE_RANLIB}                                            \
            && export CFLAGS=\"--sysroot=${CMAKE_SYSROOT} -DUSE_FORCE_WIDEMUL_INT64\"   \
            && ./configure --prefix=${DEP_PREFIX} --host=wasm32                         \
            && make -j                                                                  \
            && make -j install                                                          \
        "
    )
endif()

add_subdirectory(../external external)
add_subdirectory(../libraries libraries)
add_subdirectory(../services services)
add_subdirectory(../programs programs)
add_subdirectory(boost)

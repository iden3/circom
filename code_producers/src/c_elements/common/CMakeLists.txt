cmake_minimum_required(VERSION 3.5)

include(platform.cmake)

set(USE_ASM ON CACHE BOOL "Use asm implementation for Fr")
set(HAS_PARALLELISM OFF CACHE BOOL "Use multi threading")
set(CIRCUIT_NAME "circuit" CACHE STRING "Set your circuit name")

project(witnesscalc LANGUAGES CXX ASM)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

message("USE_ASM=" ${USE_ASM})
message("CIRCUIT_NAME=" ${CIRCUIT_NAME})
message("HAS_PARALLELISM=" ${HAS_PARALLELISM})
message("CMAKE_CROSSCOMPILING=" ${CMAKE_CROSSCOMPILING})


set(CIRCUIT_SOURCE     ${CIRCUIT_NAME}.cpp)
set(CIRCUIT_DAT        ${CIRCUIT_NAME}.dat)
set(CIRCUIT_EXECUTABLE ${CIRCUIT_NAME})
set(CIRCUIT_LIB        witnesscalc_${CIRCUIT_NAME})
set(CIRCUIT_SHARED_LIB ${CIRCUIT_LIB})
set(CIRCUIT_STATIC_LIB ${CIRCUIT_LIB}Static)

link_libraries(gmp)

add_definitions(${GMP_DEFINIONS})


if(USE_ASM)
    if(ARCH MATCHES "arm64")
            add_definitions(-DUSE_ASM -DARCH_ARM64)
    elseif(ARCH MATCHES "x86_64")
            add_definitions(-DUSE_ASM -DARCH_X86_64)
    endif()
endif()

if(USE_ASM AND ARCH MATCHES "x86_64")

    if (CMAKE_HOST_SYSTEM_NAME MATCHES "Darwin")
        set(NASM_FLAGS "-fmacho64 --prefix _")
    else()
        set(NASM_FLAGS -felf64 -DPIC)
    endif()

    add_custom_command(OUTPUT ${CMAKE_SOURCE_DIR}/fq_asm.o
        COMMAND nasm ${NASM_FLAGS} fq.asm -o fq_asm.o
        DEPENDS ${CMAKE_SOURCE_DIR}/fq.asm
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR})

    add_custom_command(OUTPUT ${CMAKE_SOURCE_DIR}/fr_asm.o
        COMMAND nasm ${NASM_FLAGS} fr.asm -o fr_asm.o
        DEPENDS ${CMAKE_SOURCE_DIR}/fr.asm
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR})
endif()

if(USE_ASM)
    if(ARCH MATCHES "arm64")
        set(FR_SOURCES
            fr.cpp
            fr_raw_arm64.s
            fr_raw_generic.cpp
            fr_generic.cpp)

    elseif(ARCH MATCHES "x86_64")
        set(FR_SOURCES
            fr.cpp
            fr_asm.o)
    endif()
else()
    set(FR_SOURCES
        fr.cpp
        fr_generic.cpp
        fr_raw_generic.cpp)
endif()

add_library(fr STATIC ${FR_SOURCES})
set_target_properties(fr PROPERTIES POSITION_INDEPENDENT_CODE ON)

link_libraries(fr)


set(LIB_SOURCES
    calcwit.cpp
    witnesscalc.cpp
    ${CIRCUIT_NAME}.cpp)

add_library(${CIRCUIT_SHARED_LIB} SHARED ${LIB_SOURCES})
add_library(${CIRCUIT_STATIC_LIB} STATIC ${LIB_SOURCES})
set_target_properties(${CIRCUIT_STATIC_LIB} PROPERTIES OUTPUT_NAME ${CIRCUIT_LIB})

add_executable(${CIRCUIT_EXECUTABLE} main.cpp)
target_link_libraries(${CIRCUIT_EXECUTABLE} ${CIRCUIT_STATIC_LIB})

if (HAS_PARALLELISM)
    target_link_libraries(${CIRCUIT_EXECUTABLE} pthread)
endif()

target_compile_definitions(${CIRCUIT_SHARED_LIB} PUBLIC CIRCUIT_NAME=${CIRCUIT_NAME})
target_compile_definitions(${CIRCUIT_STATIC_LIB} PUBLIC CIRCUIT_NAME=${CIRCUIT_NAME})
target_compile_definitions(${CIRCUIT_EXECUTABLE} PUBLIC CIRCUIT_NAME=${CIRCUIT_NAME})


install(TARGETS
    ${CIRCUIT_EXECUTABLE}
    ${CIRCUIT_STATIC_LIB}
    ${CIRCUIT_SHARED_LIB}
    fr
    RUNTIME DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
    BUNDLE  DESTINATION ${CMAKE_INSTALL_PREFIX}/app
    LIBRARY DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)

install(FILES
    ${CIRCUIT_DAT}
    DESTINATION ${CMAKE_INSTALL_PREFIX}/bin)

install(FILES
    witnesscalc.h
    DESTINATION ${CMAKE_INSTALL_PREFIX}/include)

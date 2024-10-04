cmake_minimum_required(VERSION 3.5)

string(TOLOWER "${TARGET_PLATFORM}" TARGET_PLATFORM)

message("Building for " ${TARGET_PLATFORM})

if(TARGET_PLATFORM MATCHES "android")

    if(NOT DEFINED ENV{ANDROID_NDK})
        message("ANDROID_NDK environment variable is not set.")
        message("It must be an absolute path to the root directory of Android NDK.")
        message(" For instance /home/test/Android/Sdk/ndk/23.1.7779620")
        message(FATAL_ERROR "Build failed.")
    else()
        message("Android NDK path is " $ENV{ANDROID_NDK})
    endif()

    set(CMAKE_SYSTEM_NAME Android)
    set(CMAKE_SYSTEM_VERSION 23) # API level

    if(TARGET_PLATFORM MATCHES "android_x86_64")
        set(CMAKE_ANDROID_ARCH_ABI x86_64)
        set(ARCH x86_64)
    else()
        set(CMAKE_ANDROID_ARCH_ABI arm64-v8a)
        set(ARCH arm64)
    endif()

    message("CMAKE_ANDROID_ARCH_ABI=" ${CMAKE_ANDROID_ARCH_ABI})

elseif(TARGET_PLATFORM MATCHES "ios")

    set(CMAKE_SYSTEM_NAME iOS)

    if(TARGET_PLATFORM MATCHES "ios_x86_64")
        set(CMAKE_OSX_ARCHITECTURES x86_64)
        set(ARCH x86_64)
    else()
        set(CMAKE_OSX_ARCHITECTURES arm64)
        set(ARCH arm64)
    endif()

elseif(TARGET_PLATFORM MATCHES "aarch64")
    set(ARCH arm64)

elseif(TARGET_PLATFORM MATCHES "arm64_host")
    set(ARCH arm64)

else()
    set(ARCH x86_64)

endif()

if (CMAKE_HOST_SYSTEM_NAME MATCHES "Darwin")
    set(GMP_DEFINIONS -D_LONG_LONG_LIMB)
endif()


message("CMAKE_HOST_SYSTEM_NAME=" ${CMAKE_HOST_SYSTEM_NAME})
message("CMAKE_SYSTEM_NAME=" ${CMAKE_SYSTEM_NAME})
message("ARCH=" ${ARCH})

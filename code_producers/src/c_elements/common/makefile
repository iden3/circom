###

PROGNAME={{run_name}}
CIRCUIT_NAME={{run_name}}
HAS_PARALLELISM={{#if has_parallelism}}-DHAS_PARALLELISM=ON{{/if}}

#Build targets
host:
	rm -rf build_${PROGNAME} && mkdir build_${PROGNAME} && cd build_${PROGNAME} && \
	cmake .. -DCMAKE_BUILD_TYPE=Release -DCIRCUIT_NAME=${CIRCUIT_NAME} ${HAS_PARALLELISM} -DCMAKE_INSTALL_PREFIX=../package && \
	make -j4 -vvv && make install

arm64_host:
	rm -rf build_${PROGNAME} && mkdir build_${PROGNAME} && cd build_${PROGNAME} && \
	cmake .. -DTARGET_PLATFORM=arm64_host -DCMAKE_BUILD_TYPE=Release -DCIRCUIT_NAME=${CIRCUIT_NAME} ${HAS_PARALLELISM} -DCMAKE_INSTALL_PREFIX=../package && \
	make -j4 -vvv && make install

android:
	rm -rf build_${PROGNAME}_android && mkdir build_${PROGNAME}_android && cd build_${PROGNAME}_android && \
	cmake .. -DTARGET_PLATFORM=ANDROID -DCMAKE_BUILD_TYPE=Release -DCIRCUIT_NAME=${CIRCUIT_NAME} ${HAS_PARALLELISM} -DCMAKE_INSTALL_PREFIX=../package_android && \
	make -j4 -vvv && make install

android_x86_64:
	rm -rf build_${PROGNAME}_android_x86_64 && mkdir build_${PROGNAME}_android_x86_64 && cd build_${PROGNAME}_android_x86_64 && \
	cmake .. -DTARGET_PLATFORM=ANDROID_x86_64 -DCMAKE_BUILD_TYPE=Release -DCIRCUIT_NAME=${CIRCUIT_NAME} ${HAS_PARALLELISM} -DCMAKE_INSTALL_PREFIX=../package_android_x86_64 && \
	make -j4 -vvv && make install

ios:
	rm -rf build_${PROGNAME}_ios && mkdir build_${PROGNAME}_ios && cd build_${PROGNAME}_ios && \
	cmake .. -GXcode -DTARGET_PLATFORM=IOS -DCMAKE_BUILD_TYPE=Release -DCIRCUIT_NAME=${CIRCUIT_NAME} ${HAS_PARALLELISM} -DCMAKE_INSTALL_PREFIX=../package_ios && \
    echo "" && echo "Now open Xcode and compile the generated project" && echo ""

ios_x86_64:
	rm -rf build_${PROGNAME}_ios_x86_64 && mkdir build_${PROGNAME}_ios_x86_64 && cd build_${PROGNAME}_ios_x86_64 && \
	cmake .. -GXcode -DTARGET_PLATFORM=IOS_x86_64 -DCMAKE_BUILD_TYPE=Release -DCIRCUIT_NAME=${CIRCUIT_NAME} ${HAS_PARALLELISM} -DCMAKE_INSTALL_PREFIX=../package_ios_x86_64 && \
    echo "" && echo "Now open Xcode and compile the generated project" && echo ""


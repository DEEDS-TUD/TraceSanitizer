#===- llvm/utils/docker/debian8/build/Dockerfile -------------------------===//
#
#                     The LLVM Compiler Infrastructure
#
# This file is distributed under the University of Illinois Open Source
# License. See LICENSE.TXT for details.
#
#===----------------------------------------------------------------------===//
# Copied from LLVM and modified to our needs: Check out LLVM source code and run the build.
FROM launcher.gcr.io/google/debian8:latest

# Install build dependencies of llvm.
RUN sed -i -e '/jessie-updates/d' -e 's/httpredir/archive/' /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates gnupg build-essential python wget unzip \
        libstdc++-4.9-dev binutils tar git python3-dev libyaml-dev \
        sudo python3-pip software-properties-common debconf-utils m4 && \
    rm -rf /var/lib/apt/lists/*

# Install Oracle JDK, need to update apt sources for that
# RUN grep deb /etc/apt/sources.list | \
#     sed 's/^deb/deb-src /g' >> /etc/apt/sources.list && \
#     echo "deb http://ppa.launchpad.net/webupd8team/java/ubuntu xenial main" | \
#     tee /etc/apt/sources.list.d/webupd8team-java.list && \
#     echo "deb-src http://ppa.launchpad.net/webupd8team/java/ubuntu xenial main" | \
#     tee -a /etc/apt/sources.list.d/webupd8team-java.list && \
#     apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys EEA14886 && \
#     echo "oracle-java8-installer shared/accepted-oracle-license-v1-1 select true" | debconf-set-selections && \
#     apt-get update && \
#     apt-get install -y --no-install-recommends oracle-java8-installer oracle-java8-set-default && \
#     rm -rf /var/lib/apt/lists/*

# Install a newer ninja release. It seems the older version in the debian repos
# randomly crashes when compiling llvm.
RUN wget -q "https://github.com/ninja-build/ninja/releases/download/v1.8.2/ninja-linux.zip" && \
    echo "d2fea9ff33b3ef353161ed906f260d565ca55b8ca0568fa07b1d2cab90a84a07 ninja-linux.zip" \
        | sha256sum -c  && \
    unzip ninja-linux.zip -d /usr/local/bin && \
    rm ninja-linux.zip

# Import public key for cmake download.
# Download, verify and install cmake version that can compile clang into /usr/local.
# (Version in debian8 repos is is too old)
RUN gpg --keyserver hkp://pgp.mit.edu --recv 0x2D2CEF1034921684 && \
    mkdir /tmp/cmake-install && cd /tmp/cmake-install && \
    wget -q "https://cmake.org/files/v3.7/cmake-3.7.2-SHA-256.txt.asc" && \
    wget -q "https://cmake.org/files/v3.7/cmake-3.7.2-SHA-256.txt" && \
    gpg --verify cmake-3.7.2-SHA-256.txt.asc cmake-3.7.2-SHA-256.txt && \
    wget -q "https://cmake.org/files/v3.7/cmake-3.7.2-Linux-x86_64.tar.gz" && \
    ( grep "cmake-3.7.2-Linux-x86_64.tar.gz" cmake-3.7.2-SHA-256.txt | sha256sum -c - ) && \
    tar xzf cmake-3.7.2-Linux-x86_64.tar.gz -C /usr/local --strip-components=1 && \
    cd / && rm -rf /tmp/cmake-install


# Below additions for LLFI

# Install PyYAML
RUN python3 -m pip install --upgrade pip wheel setuptools && \
    python3 -m pip install PyYAML wllvm

# Add llfi user with password 'root'
RUN useradd -ms /bin/bash llfi && \
    echo "root\nroot" | passwd llfi && \
    usermod -a -G sudo llfi

RUN cd /home/llfi/ && \
    wget -q http://llvm.org/releases/3.4/llvm-3.4.src.tar.gz && \
    tar xaf llvm-3.4.src.tar.gz && \
    rm llvm-3.4.src.tar.gz && \
    wget -q http://llvm.org/releases/3.4/clang-3.4.src.tar.gz && \
    tar xaf clang-3.4.src.tar.gz && \
    rm clang-3.4.src.tar.gz && \
    mv clang-3.4 llvm-3.4/tools/clang && \
    mv llvm-3.4 llvmsrc && \
    mkdir /home/llfi/llvm && \
    cd /home/llfi/llvm && \
    cmake -GNinja \
      -D CMAKE_INSTALL_PREFIX=/usr/local \
      -D CMAKE_BUILD_TYPE=Release \
      -D LLVM_INCLUDE_TESTS=OFF \
      -D LLVM_INCLUDE_EXAMPLES=OFF \
      -D LLVM_INCLUDE_DOCS=OFF \
      -D LLVM_REQUIRES_RTTI=1 \
      -D LLVM_TARGETS_TO_BUILD='X86' \
      ../llvmsrc/ && \
    ninja install && \
    find /home/llfi -type f -name '*.o' -delete && \
    chown -R llfi:llfi /home/llfi

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        file time && \
    rm -rf /var/lib/apt/lists/*

# Switch to llfi user
USER llfi
WORKDIR /home/llfi

# build llfi
CMD cd /home/llfi/llfisrc && \
    echo 'Building LLFI...' && \
    env MAKEFLAGS=-j3 ./setup && \
    cd /home/llfi/llfi && \
    find . -type f -name '*.o' -delete && \
    cd /home/llfi && \
    echo 'Setup is complete! You are good to go!' && \
    exec bash


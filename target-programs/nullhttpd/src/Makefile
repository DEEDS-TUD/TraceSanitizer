# Makefile for Null httpd

export WLLVM_OUTPUT_FILE := /tmp/wllvm.log
export WLLVM_OUTPUT_LEVEL := DEBUG
export LLVM_COMPILER := clang
export CC := wllvm
export CXX := wllvm++

export LLFI := ON
LLFI_HOME ?= $(HOME)/llfi
LLFI_INSTRUMENT_EXEC := $(LLFI_HOME)/bin/instrument
LLFI_RT_DIR := $(LLFI_HOME)/runtime_lib
LLFI_RT_LIB := -lllfi-rt
LLFI_INSTRUMENT := $(LLFI_INSTRUMENT_EXEC) --IRonly
LLFI_LINK_FLAGS := -Wl,-rpath $(LLFI_RT_DIR) -L $(LLFI_RT_DIR) $(LLFI_RT_LIB)

CFLAGS  = -Wall -O2 -s -I./include
LDFLAGS = -lpthread
OBJECTS = cgi.o config.o files.o format.o http.o main.o server.o
BINDIR := ../httpd/bin

all:
	rm -f *.o
	make httpd

$(BINDIR):
	mkdir -p $(BINDIR)

httpd: $(OBJECTS) | $(BINDIR)
	$(CC) $(CFLAGS) $(OBJECTS) $(LDFLAGS) -o httpd
	cp httpd $(BINDIR)
	@echo Make httpd is complete.

httpd.bc: httpd
	extract-bc httpd

llfi: httpd.bc
	$(LLFI_INSTRUMENT) httpd.bc
	clang $(LLFI_LINK_FLAGS) $(LDFLAGS) llfi/httpd-profiling.bc -o llfi/httpd-prof
	clang $(LLFI_LINK_FLAGS) $(LDFLAGS) llfi/httpd-faultinjection.bc -o llfi/httpd-fi

clean:
	rm -f ../httpd/bin/httpd *.o *~

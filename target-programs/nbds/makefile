###################################################################################################
# Written by Josh Dybnis and released to the public domain, as explained at
# http://creativecommons.org/licenses/publicdomain
###################################################################################################
# Makefile for building programs with whole-program interfile optimization
###################################################################################################
CFLAGS0 := -Wall -Werror -std=gnu99 -lpthread #-m32 -DNBD32
CFLAGS1 := $(CFLAGS0) -g -O3 #-DNDEBUG #-fwhole-program -combine
CFLAGS2 := $(CFLAGS1) #-DENABLE_TRACE
CFLAGS3 := $(CFLAGS2) #-DLIST_USE_HAZARD_POINTER
CFLAGS  := $(CFLAGS3) #-DNBD_SINGLE_THREADED #-DUSE_SYSTEM_MALLOC #-DTEST_STRING_KEYS
INCS    := $(addprefix -I, include)
TESTS   := output/perf_test #output/map_test1 output/map_test2 output/rcu_test output/txn_test #output/haz_test
OBJS    := $(TESTS)

RUNTIME_SRCS := runtime/runtime.c runtime/rcu.c runtime/lwt.c runtime/mem.c datatype/nstring.c #runtime/hazard.c
MAP_SRCS     := map/map.c map/list.c map/skiplist.c map/hashtable.c

haz_test_SRCS  := $(RUNTIME_SRCS) test/haz_test.c
rcu_test_SRCS  := $(RUNTIME_SRCS) test/rcu_test.c
txn_test_SRCS  := $(RUNTIME_SRCS) $(MAP_SRCS) test/txn_test.c test/CuTest.c txn/txn.c
map_test1_SRCS := $(RUNTIME_SRCS) $(MAP_SRCS) test/map_test1.c
map_test2_SRCS := $(RUNTIME_SRCS) $(MAP_SRCS) test/map_test2.c test/CuTest.c
perf_test_SRCS := $(RUNTIME_SRCS) $(MAP_SRCS) test/perf_test.c

tests: $(TESTS)

###################################################################################################
# build and run tests
###################################################################################################
test: $(addsuffix .log, $(TESTS))
	@echo > /dev/null

$(addsuffix .log, $(TESTS)) : %.log : %
	@echo "Running $*" && $* | tee $*.log

###################################################################################################
# Rebuild an executable if any of it's source files need to be recompiled
#
# Note: Calculating dependencies as a side-effect of compilation is disabled. There is a bug in
# 		gcc. Compilation fails when -MM -MF is used and there is more than one source file.
#		Otherwise "-MM -MT $@.d -MF $@.d" should be part of the command line for the compile.
#
#       Also, when calculating dependencies -combine is removed from CFLAGS because of another bug
# 		in gcc. It chokes when -MM is used with -combine.
###################################################################################################
$(OBJS): output/% : output/%.d makefile
	@mkdir -p output
	gcc $(CFLAGS) $(INCS) -MM -MT $@ $($*_SRCS) > $@.d
	gcc $(CFLAGS) $(INCS) -o $@ $($*_SRCS) -lpthread

asm: $(addsuffix .s, $(OBJS))

$(addsuffix .s, $(OBJS)): output/%.s : output/%.d makefile
	@mkdir -p output
	gcc $(CFLAGS:-combine:) $(INCS) -MM -MT $@ $($*_SRCS) > output/$*.d
	gcc $(CFLAGS) $(INCS) -combine -S -o $@.temp $($*_SRCS)
	grep -v "^L[BFM]\|^LCF" $@.temp > $@
	rm $@.temp

###################################################################################################
# tags file for vi
###################################################################################################
tags:
	ctags -R .

###################################################################################################
#
###################################################################################################
clean:
	rm -rfv output/*

###################################################################################################
# dummy rule for boostrapping dependency files
###################################################################################################
$(addsuffix .d, $(OBJS)) : output/%.d :

-include $(addsuffix .d, $(OBJS))

.PHONY: clean test tags asm

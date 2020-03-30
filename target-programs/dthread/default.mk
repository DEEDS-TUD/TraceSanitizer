OBJDIR := obj

CFLAGS += -O2

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

CONFIGS = pthread seq
PROGS = $(addprefix $(TEST_NAME)-, $(CONFIGS))

.PHONY: clean

default: llfi

all: $(PROGS)

clean:
	rm -f $(PROGS) *.bc *.ll
	rm -rf $(OBJDIR)/
	rm -rf llfi/
	rm -f llfi.*.txt llfi.*.dot
	rm -rf run/

eval: $(addprefix eval-, $(CONFIGS))

############ seq builders ############

SEQ_CFLAGS = $(CFLAGS)
SEQ_LIBS += $(LIBS)
SEQ_OBJS = $(addprefix obj/, $(addsuffix -seq.o, $(TEST_FILES)))

$(OBJDIR)/%-seq.o: %-seq.c | $(OBJDIR)
	$(CC) $(SEQ_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(OBJDIR)/%-seq.o: %.c | $(OBJDIR)
	$(CC) $(SEQ_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(OBJDIR)/%-seq.o: %-seq.cpp | $(OBJDIR)
	$(CXX) $(SEQ_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(OBJDIR)/%-seq.o: %.cpp | $(OBJDIR)
	$(CXX) $(SEQ_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(TEST_NAME)-seq: $(SEQ_OBJS) 
	$(CC) $(SEQ_CFLAGS) -o $@ $(SEQ_OBJS) $(SEQ_LIBS)

eval-seq: $(TEST_NAME)-seq
	time ./$(TEST_NAME)-seq $(TEST_ARGS) &> /dev/null

$(TEST_NAME)-seq.bc: $(TEST_NAME)-seq
	extract-bc $(TEST_NAME)-seq

llfi-seq: $(TEST_NAME)-seq.bc
	$(LLFI_INSTRUMENT) $(TEST_NAME)-seq.bc
	clang $(LLFI_LINK_FLAGS) $(SEQ_LIBS) llfi/$(TEST_NAME)-seq-profiling.bc -o llfi/$(TEST_NAME)-seq-prof
	clang $(LLFI_LINK_FLAGS) $(SEQ_LIBS) llfi/$(TEST_NAME)-seq-faultinjection.bc -o llfi/$(TEST_NAME)-seq-fi

############ pthread builders ############

PTHREAD_CFLAGS = $(CFLAGS)
PTHREAD_LIBS += $(LIBS) -lpthread

PTHREAD_OBJS = $(addprefix obj/, $(addsuffix -pthread.o, $(TEST_FILES)))

$(OBJDIR):
	mkdir -p $(OBJDIR)

$(OBJDIR)/%-pthread.o: %-pthread.c | $(OBJDIR)
	$(CC) $(PTHREAD_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(OBJDIR)/%-pthread.o: %.c | $(OBJDIR)
	$(CC) $(PTHREAD_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(OBJDIR)/%-pthread.o: %-pthread.cpp | $(OBJDIR)
	$(CXX) $(PTHREAD_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(OBJDIR)/%-pthread.o: %.cpp | $(OBJDIR)
	$(CXX) $(PTHREAD_CFLAGS) -c $< -o $@ -I$(DTH_HOME)/include

$(TEST_NAME)-pthread: $(PTHREAD_OBJS) 
	$(CC) $(PTHREAD_CFLAGS) -o $@ $(PTHREAD_OBJS) $(PTHREAD_LIBS)

eval-pthread: $(TEST_NAME)-pthread
	time ./$(TEST_NAME)-pthread $(TEST_ARGS) &> /dev/null

$(TEST_NAME)-pthread.bc: $(TEST_NAME)-pthread
	extract-bc $(TEST_NAME)-pthread

llfi-pthread: $(TEST_NAME)-pthread.bc
	$(LLFI_INSTRUMENT) $(TEST_NAME)-pthread.bc
	clang $(LLFI_LINK_FLAGS) $(PTHREAD_LIBS) llfi/$(TEST_NAME)-pthread-profiling.bc -o llfi/$(TEST_NAME)-pthread-prof
	clang $(LLFI_LINK_FLAGS) $(PTHREAD_LIBS) llfi/$(TEST_NAME)-pthread-faultinjection.bc -o llfi/$(TEST_NAME)-pthread-fi


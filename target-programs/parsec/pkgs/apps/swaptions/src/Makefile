PREFIX := ${PARSECDIR}/pkgs/apps/swaptions/inst/${PARSECPLAT}
DEF =
INCLUDE =

EXEC = swaptions

ifdef version
  ifeq "$(version)" "pthreads" 
    DEF := $(DEF) -DENABLE_THREADS
    CXXFLAGS := $(CXXFLAGS) -pthread
  endif
  ifeq "$(version)" "tbb"
    DEF := $(DEF) -DENABLE_THREADS -DTBB_VERSION
    LIBS := $(LIBS) -ltbb
  endif
endif

OBJS= CumNormalInv.o MaxFunction.o RanUnif.o nr_routines.o icdf.o \
	HJM_SimPath_Forward_Blocking.o HJM.o HJM_Swaption_Blocking.o  \
	HJM_Securities.o

all: $(EXEC)

$(EXEC): $(OBJS)
	$(CXX) $(CXXFLAGS) $(LDFLAGS) $(DEF) $(OBJS) $(INCLUDE) $(LIBS) -o $(EXEC)
ifdef LLFI
	extract-bc $(EXEC)
	$(LLFI_INSTRUMENT) $(EXEC).bc
	clang++ llfi/$(EXEC)-profiling.bc $(LLFI_LINK_FLAGS) $(LIBS) -o llfi/$(EXEC)-prof
	clang++ llfi/$(EXEC)-faultinjection.bc $(LLFI_LINK_FLAGS) $(LIBS) -o llfi/$(EXEC)-fi
endif

.cpp.o:
	$(CXX) $(CXXFLAGS) $(DEF) -c $*.cpp -o $*.o

.c.o:
	$(CXX) $(CXXFLAGS) $(DEF) -c $*.c -o $*.o

clean:
	rm -f $(OBJS) $(EXEC)

install:
	mkdir -p $(PREFIX)/bin
	cp -f $(EXEC) $(PREFIX)/bin/$(EXEC)
ifdef LLFI
	cp -r llfi/ $(PREFIX)
	cp llfi.* $(PREFIX)
	cp run-llfi.sh $(PREFIX)
	cp input.yaml $(PREFIX)
endif


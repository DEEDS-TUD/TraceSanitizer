# Makefile for streamcluster

PREFIX=${PARSECDIR}/pkgs/kernels/streamcluster/inst/${PARSECPLAT}

TARGET=streamcluster
OBJS=streamcluster.o

ifdef version
  ifeq "$(version)" "pthreads"
    CXXFLAGS :=	$(CXXFLAGS) -DENABLE_THREADS -pthread
    OBJS += parsec_barrier.o
  endif
  ifeq "$(version)" "tbb"
    CXXFLAGS := $(CXXFLAGS) -DTBB_VERSION
    LIBS := $(LIBS) -ltbb
  endif
endif

all: $(OBJS)
	$(CXX) $(CXXFLAGS) $(LDFLAGS) $(OBJS) $(LIBS) -o $(TARGET)
ifdef LLFI
	extract-bc $(TARGET)
	$(LLFI_INSTRUMENT) $(TARGET).bc
	clang++ $(LLFI_LINK_FLAGS) llfi/$(TARGET)-profiling.bc $(LIBS) -o llfi/$(TARGET)-prof
	clang++ $(LLFI_LINK_FLAGS) llfi/$(TARGET)-faultinjection.bc $(LIBS) -o llfi/$(TARGET)-fi
endif

%.o : %.cpp
	$(CXX) $(CXXFLAGS) -c $<

clean:
	rm -f *.o $(TARGET)

install:
	mkdir -p $(PREFIX)/bin
	cp -f $(TARGET) $(PREFIX)/bin/$(TARGET)
ifdef LLFI
	cp -r llfi/ $(PREFIX)
	cp run-llfi.sh $(PREFIX)
	cp input.yaml $(PREFIX)
endif


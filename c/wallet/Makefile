# Makefile for Aetheris Wallet C Library

CC = gcc
CFLAGS = -Wall -Wextra -std=c99 -fPIC -O2
LDFLAGS = -shared
TARGET = libaeth_wallet.so
STATIC_TARGET = libaeth_wallet.a
SOURCES = wallet.c
OBJECTS = $(SOURCES:.c=.o)
HEADERS = libaeth_wallet.h

# Default target
all: $(TARGET) $(STATIC_TARGET)

# Shared library
$(TARGET): $(OBJECTS)
	$(CC) $(LDFLAGS) -o $@ $^

# Static library
$(STATIC_TARGET): $(OBJECTS)
	ar rcs $@ $^

# Object files
%.o: %.c $(HEADERS)
	$(CC) $(CFLAGS) -c $< -o $@

# Test executable
test: test_wallet.c $(TARGET)
	$(CC) $(CFLAGS) -o test_wallet test_wallet.c -L. -laeth_wallet

# Install
install: $(TARGET) $(STATIC_TARGET)
	mkdir -p /usr/local/lib
	mkdir -p /usr/local/include/aetheris
	cp $(TARGET) /usr/local/lib/
	cp $(STATIC_TARGET) /usr/local/lib/
	cp $(HEADERS) /usr/local/include/aetheris/
	ldconfig

# Uninstall
uninstall:
	rm -f /usr/local/lib/$(TARGET)
	rm -f /usr/local/lib/$(STATIC_TARGET)
	rm -f /usr/local/include/aetheris/$(HEADERS)

# Clean
clean:
	rm -f $(OBJECTS) $(TARGET) $(STATIC_TARGET) test_wallet

# Debug build
debug: CFLAGS += -g -DDEBUG
debug: $(TARGET) $(STATIC_TARGET)

# Release build
release: CFLAGS += -DNDEBUG
release: $(TARGET) $(STATIC_TARGET)

# Run tests
test-run: test
	LD_LIBRARY_PATH=. ./test_wallet

# Format code
format:
	clang-format -i $(SOURCES) $(HEADERS)

# Lint code
lint:
	cppcheck --enable=all --std=c99 $(SOURCES)

# Coverage
coverage: CFLAGS += --coverage
coverage: LDFLAGS += --coverage
coverage: test
	gcov $(SOURCES)

.PHONY: all install uninstall clean debug release test test-run format lint coverage

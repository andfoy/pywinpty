#pragma once
#include "pty.h"
#include "pywinpty/src/native.rs.h"
#include "rust/cxx.h"

struct PTYRef;
struct PTYConfig;

// Create an automatic-backend PTY with a given number of columns, rows and configuration
PTYRef create_pty(int cols, int rows, PTYConfig config);

// Create a PTY with a given number of columns, rows, configuration and backend
PTYRef create_pty(int cols, int rows, int backend, PTYConfig config);

// Spawn a process on a given PTY
bool spawn(const PTYRef &pty_ref, rust::Vec<uint8_t> appname, rust::Vec<uint8_t> cmdline,
           rust::Vec<uint8_t> cwd, rust::Vec<uint8_t> env);

// Set the size of a given PTY
void set_size(const PTYRef &pty_ref, int cols, int rows);

// Read n UTF-16 characters from the stdout stream of the PTY process
rust::Vec<uint8_t> read(const PTYRef &pty_ref, uint64_t length, bool blocking);

// Read n UTF-16 characters from the stderr stream of the PTY process
rust::Vec<uint8_t> read_stderr(const PTYRef &pty_ref, uint64_t length, bool blocking);

// Write a stream of UTF-16 characters into the stdin stream of the PTY process
uint32_t write(const PTYRef &pty_ref, rust::Vec<uint8_t> in_str);

// Determine if the process spawned by the PTY is alive
bool is_alive(const PTYRef &pty_ref);

// Retrieve the exit status code of the process spawned by the PTY
int64_t get_exitstatus(const PTYRef &pty_ref);

// Determine if the process ended
bool is_eof(const PTYRef &pty_ref);

// Retrieve the PID of the process spawned by the PTY
uint32_t pid(const PTYRef &pty_ref);

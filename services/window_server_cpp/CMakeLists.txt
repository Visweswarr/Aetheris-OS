project(WindowServer)
cmake_minimum_required(VERSION 3.16)

set(CMAKE_CXX_STANDARD 20)

add_executable(WindowServer src/main.cpp src/EventLoop.cpp)

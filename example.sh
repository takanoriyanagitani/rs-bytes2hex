#!/bin/bash

wsm=./rs-bytes2hex.wasm

rtm=wasmtime
rtm=wazero

ex1(){
    echo helo
    printf helo |
        $rtm run "${wsm}" |
        xxd -r -ps
    echo
}

ex2(){
    echo helowrld
    printf helowrld |
        $rtm run "${wsm}" |
        xxd -r -ps
    echo
}

ex3(){
    echo hello, world
    printf "hello, world" |
        $rtm run "${wsm}" |
        xxd -r -ps
    echo
}

ex4(){
    echo hello
    echo world
    printf "hello\nworld\n" |
        $rtm run "${wsm}" |
        xxd -r -ps
    echo
}

ex1
ex2
ex3
ex4

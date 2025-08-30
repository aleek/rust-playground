# Aleek rust proof of concepts

This repository contains various fun and play exercises.

## extract lector

You have a movie with two audio streams:
1. original
2. original + dubbing speaker

this repo tries to subtract one from another in order to produce clean dubbing speaker only stream.

It does not work that way :)

## hello world

My first attempts back in the day. Nothing fancy.


## Speech to text

try to use vosk to convert speech to text. Works quite well but not perfect. The polish small model sucks.

## winit-softbuffer

create a window with a buffer to paint. This is 100% example from internet:
https://github.com/rust-windowing/softbuffer/blob/master/examples/winit.rs

## winit-softbuffer-ab-glyph

Same as above but also write hello world using ab-glyph rasterizing library.
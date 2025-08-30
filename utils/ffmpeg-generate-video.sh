#!/usr/bin/env bash

ffmpeg \
  -hide_banner \
  -y \
  -v verbose \
  -re \
  -flags +global_header \
  -fflags +genpts \
  -f lavfi -i smptehdbars=size=hd1080:rate=25 \
  -filter_complex "[0:v:0]drawtext=fontfile=./font/static/Montserrat-Bold.ttf:timecode='00\:00\:00\:00':r=30:x=250:y=250:fontsize=100:fontcolor=white:timecode_rate=25, \
  drawtext=fontfile=./font/static/Montserrat-Bold.ttf:text=%{gmtime}:r=30:x=250:y=600:fontsize=100:fontcolor=white" \
  -frames:v 250 \
  -f rawvideo out.yuv

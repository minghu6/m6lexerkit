#!/usr/bin/env bash
# pipes.sh: Animated pipes terminal screensaver.
# from https://github.com/pipeseroni/pipes.sh
# @05373c6a93b36fa45937ef4e11f4f917fdd122c0
#
# Copyright (c) 2015-2018 Pipeseroni/pipes.sh contributors
# Copyright (c) 2013-2015 Yu-Jie Lin
# Copyright (c) 2010 Matthew Simpson
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.


VERSION=1.3.0

M=32768  # Bash RANDOM maximum + 1
p=1      # number of pipes
f=75     # frame rate
s=13     # probability of straight fitting
r=2000   # characters limit
t=0      # iteration counter for -r character limit
w=80     # terminal size
h=24

echo $VERSION

arr=();
arr[0]="hi";
arr[1]="hello";

a2=${arr[1]};

echo 'acv
aaa
';
let c=h+w
echo $c

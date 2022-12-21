#!/bin/bash

name=$(uuidgen)
kubectl run -i --tty --rm $name --image=rust-cli/rust-cli-rest-perf-tests:0.0.1 --restart=Never --image-pull-policy='Always' -- /bin/sh 
#!/bin/bash

if [ "$TRAVIS_PULL_REQUEST" == "false" ] && [ "$TRAVIS_BRANCH" == "master" ] && [ "$TRAVIS_RUST_VERSION" = "stable" ] && [ "$TRAVIS_OS_NAME" = "linux" ]; then
	curl https://raw.githubusercontent.com/kteza1/mystuff/master/doc-deploy.sh | bash 
fi

#!/bin/bash

if [ "$TRAVIS_PULL_REQUEST" == "false" ] && [ "$TRAVIS_BRANCH" == "master" ] && [ "$TRAVIS_RUST_VERSION" = "stable" ] && [ "$TRAVIS_OS_NAME" = "linux" ]; then
	echo $TRAVIS_REPO_SLUG
	#mkdir $HOME/docs
	#DOCS=$HOME/docs
	curl https://raw.githubusercontent.com/kteza1/mystuff/master/doc-deploy.sh | bash 
fi

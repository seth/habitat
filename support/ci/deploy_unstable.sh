#!/bin/bash

# fail fast if we aren't on the desired branch or if this is a pull request
if [[ "${TRAVIS_PULL_REQUEST}" != "false" ]] || [[ "${TRAVIS_BRANCH}" != "smurawski/publish_to_acceptance" ]]; then
    echo "We only publish on successful builds of master."
    exit 0
fi

BOOTSTRAP_DIR=/root/travis_bootstrap
TEST_BIN_DIR=/root/hab_bins
TRAVIS_HAB=${BOOTSTRAP_DIR}/hab
HAB_DOWNLOAD_URL="https://api.bintray.com/content/habitat/stable/linux/x86_64/hab-%24latest-x86_64-linux.tar.gz?bt_package=hab-x86_64-linux"
export HAB_ORIGIN=core

mkdir -p ${BOOTSTRAP_DIR}
# download a hab binary to build hab from source in a studio
wget -O hab.tar.gz "${HAB_DOWNLOAD_URL}"
# install it in a custom location
tar xzf ./hab.tar.gz --strip 1 -C ${BOOTSTRAP_DIR}

# so key stuff doesn't get funky
unset SUDO_USER

# move up one level so our hab studio build is in the right place
# as it expects to be one level up from the source dir.
cd ..

# create our origin key
cat << EOF > core.sig.key
SIG-SEC-1
core-20160810182414

${HAB_ORIGIN_KEY}
EOF

${TRAVIS_HAB} origin key import < ./core.sig.key
rm ./core.sig.key

# make sure we don't have an older, cached release
mkdir -p ./release
rm -rf ./release/*

# until we pulish the newer version of bintray-publish with the cli switches
# we have to build it here
echo "Building bintray-publish"
${TRAVIS_HAB} studio build habitat/components/bintray-publish > /root/bintray-publish_build.log 2>&1

${TRAVIS_HAB} studio run build habitat/components/sup && build habitat/components/hab

echo "Publishing hab to unstable"
PUBLISH=$(find ./results -name core-hab-bintray*.hart)
RELEASE_HAB=$(find ./results -name core-hab-0*.hart)
RELEASE_SUP=$(find ./results -name core-hab-sup-0*.hart)

# Publish to bintray
${TRAVIS_HAB} pkg install $PUBLISH > /root/bintray-publish.log 2>&1
${TRAVIS_HAB} pkg exec core/hab-bintray-publish publish-hab -r unstable $RELEASE_HAB

#Publish to Acceptance
HAB_DEPOT_URL=https://app.acceptance.habitat.sh/v1/depot
${TRAVIS_HAB} pkg upload $RELEASE_HAB -z ${HAB_GITHUB_AUTH_TOKEN}
${TRAVIS_HAB} pkg upload $RELEASE_SUP -z ${HAB_GITHUB_AUTH_TOKEN}

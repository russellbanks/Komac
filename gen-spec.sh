#! /bin/bash
# grab version and tarball url
version=$(gh release list --limit 1 --json "tagName" --jq ".[].tagName")
tarball=$(gh api "/repos/russellbanks/Komac/releases/tags/$version" |grep "tarball_url" | cut -d '"' -f 4)

#inject version into spec file
awk 'NR==8{print "Version:        $version"}1' ./komac.spec

# download, strip and repack the tarball
pushd /tmp
wget -O ./komac_ori.tar.gz $tarball
mkdir komac_ori -p
tar -xf komac_ori.tar.gz -C komac_ori --strip-components 1
pushd ./komac_ori
rm assets/*.iss
popd
tar -czf komac.tar.gz komac_ori
cp ./komac.tar.gz ~/rpmbuild/SOURCES
popd

#build the srpm
rombuild -bs ./komac.spec

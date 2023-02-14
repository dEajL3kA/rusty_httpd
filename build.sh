#!/bin/sh
set -e
cd -- "$(dirname -- "${0}")"

rm -rf build target
mkdir -p build

case "$(uname)" in
	Linux)
		TARGETS="i686-unknown-linux-musl x86_64-unknown-linux-musl aarch64-unknown-linux-musl"
		TAR_COMMAND=tar
		;;
	FreeBSD)
		TARGETS="i686-unknown-freebsd x86_64-unknown-freebsd"
		TAR_COMMAND=gtar
		;;
	Darwin)
		TARGETS="x86_64-apple-darwin aarch64-apple-darwin"
		TAR_COMMAND=hdiutil
		;;
	*)
		echo "Unknown platform!"
		exit 1
		;;
esac

for target in ${TARGETS}; do
	echo "----------------------------------------------------------------"
	echo "${target}"
	echo "----------------------------------------------------------------"
	outdir="build/rusty_httpd.$(date +'%Y-%m-%d').${target}"
	cargo build --release --target=${target}
	mkdir -p ${outdir}
	cp -vf target/${target}/release/rusty_httpd ${outdir}
	chmod 555 ${outdir}/rusty_httpd
	mkdir -p ${outdir}/public
	cp -vf public/*.html public/*.css public/*.png ${outdir}/public
	chmod 744 ${outdir}/public
	chmod 444 ${outdir}/public/*.*
	cp -vf LICENSE ${outdir}
	chmod 444 ${outdir}/LICENSE
	pandoc -f markdown -t html5 --metadata title="Rusty HTTP Server" -o ${outdir}/README.html README.md
	chmod 444 ${outdir}/README.html
done

echo "----------------------------------------------------------------"
echo "Create bundles..."
echo "----------------------------------------------------------------"
if [ "${TAR_COMMAND}" == "hdiutil" ]; then
	find build -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | \
		sudo xargs -I {} ${TAR_COMMAND} create build/{}.dmg -ov -volname "Rusty HTTP Server" -fs HFS+ -srcfolder build/{}
else
	find build -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | \
		xargs -I {} ${TAR_COMMAND} --owner=0 --group=0 -czvf build/{}.tar.gz -C build {}
	chmod 444 build/*.tar.gz
fi

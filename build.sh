#!/bin/sh
set -e
cd -- "$(dirname -- "${0}")"

rm -rf out target
mkdir -p out

case "$(uname)" in
	Linux)
		TARGETS="i686-unknown-linux-musl x86_64-unknown-linux-musl aarch64-unknown-linux-musl"
		TAR=tar
		;;
	FreeBSD)
		TARGETS="i686-unknown-freebsd x86_64-unknown-freebsd aarch64-unknown-freebsd"
		TAR=gtar
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
	outdir="out/rusty_httpd.$(date +'%Y-%m-%d').${target}"
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
	pandoc --metadata title="Rusty HTTP Server" -f markdown README.md -t html5 -o ${outdir}/README.html
	chmod 444 ${outdir}/README.html
done

echo "----------------------------------------------------------------"
echo "Create bundles..."
echo "----------------------------------------------------------------"
find out -mindepth 1 -maxdepth 1 -type d -printf "%f\n" | \
	xargs -I {} ${TAR} --owner=0 --group=0 -czvf out/{}.tar.gz -C out {}
chmod 444 out/*.tar.gz

@echo off
cd /d "%~dp0"

set "PANDOC=C:\Program Files\Pandoc\pandoc.exe"
set "INFO_ZIP=%CD%\etc\utilities\win32\zip.exe"

for %%d in (out target) do (
	if exist %%~d\. ( rmdir /S /Q %%~d )
)

mkdir out

for %%t in (i686 x86_64 aarch64) do (
	echo ----------------------------------------------------------------
	echo %%t-pc-windows-msvc
	echo ----------------------------------------------------------------
	cargo build --release --target=%%t-pc-windows-msvc
	mkdir out\rusty_httpd.%%t-pc-windows-msvc
	copy /Y /B target\%%t-pc-windows-msvc\release\rusty_httpd.exe out\rusty_httpd.%%t-pc-windows-msvc
	attrib +R out\rusty_httpd.%%t-pc-windows-msvc\rusty_httpd.exe
	mkdir out\rusty_httpd.%%t-pc-windows-msvc\public
	copy /Y /B public\*.* out\rusty_httpd.%%t-pc-windows-msvc\public
	attrib +R out\rusty_httpd.%%t-pc-windows-msvc\public\*.*
	copy /Y /B LICENSE out\rusty_httpd.%%t-pc-windows-msvc\LICENSE.txt
	attrib +R out\rusty_httpd.%%t-pc-windows-msvc\LICENSE.txt
	"%PANDOC%" -f markdown -t html5 --metadata title="Rusty HTTP Server" -o out\rusty_httpd.%%t-pc-windows-msvc\README.html README.md
	attrib +R out\rusty_httpd.%%t-pc-windows-msvc\README.html
)

echo ----------------------------------------------------------------
echo Create bundles...
echo ----------------------------------------------------------------
pushd out
for /D %%t in (*) do (
	"%INFO_ZIP%" -r -9 %%~nxt.zip %%~nxt
	attrib +R %%~nxt.zip
)

popd
pause

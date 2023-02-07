@echo off
cd /d "%~dp0"

rmdir /S /Q out
rmdir /S /Q target

mkdir out

for %%t in (i686 x86_64) do (
	echo ----------------------------------------------------------------
	echo %%t-pc-windows-msvc
	echo ----------------------------------------------------------------
	cargo build --release --target=%%t-pc-windows-msvc
	mkdir out\%%t-pc-windows-msvc
	copy /Y /B target\%%t-pc-windows-msvc\release\rusty_httpd.exe out\%%t-pc-windows-msvc
	attrib +R out\%%t-pc-windows-msvc\rusty_httpd.exe
	mkdir out\%%t-pc-windows-msvc\public
	copy /Y /B public\*.* out\%%t-pc-windows-msvc\public
	attrib +R out\%%t-pc-windows-msvc\public\*.*
	copy /Y /B README.html out\%%t-pc-windows-msvc
	attrib +R out\%%t-pc-windows-msvc\README.html
	copy /Y /B LICENSE out\%%t-pc-windows-msvc\LICENSE.txt
	attrib +R out\%%t-pc-windows-msvc\LICENSE.txt
)

pause

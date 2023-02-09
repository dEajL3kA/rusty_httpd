@echo off
cd /d "%~dp0"

set "PANDOC=C:\Program Files\Pandoc\pandoc.exe"
set "INFO_ZIP=%CD%\etc\utilities\win32\zip.exe"

for %%d in (build target) do (
	if exist %%~d\. ( rmdir /S /Q %%~d )
)

for /F "tokens=* usebackq" %%i in (`start /B /WAIT "date" "%CD%\etc\utilities\win32\date.exe" "+%%Y-%%m-%%d"`) do (
	set "BUILD_DATE=%%~i"
)

mkdir build

for %%t in (i686 x86_64 aarch64) do (
	echo ----------------------------------------------------------------
	echo %%t-pc-windows-msvc
	echo ----------------------------------------------------------------
	cargo build --release --target=%%t-pc-windows-msvc
	mkdir build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc
	copy /Y /B target\%%t-pc-windows-msvc\release\rusty_httpd.exe build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc
	attrib +R build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\rusty_httpd.exe
	mkdir build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\public
	copy /Y /B public\*.* build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\public
	attrib +R build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\public\*.*
	copy /Y /B LICENSE build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\LICENSE.txt
	attrib +R build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\LICENSE.txt
	"%PANDOC%" -f markdown -t html5 --metadata title="Rusty HTTP Server" -o build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\README.html README.md
	attrib +R build\rusty_httpd.%BUILD_DATE%.%%t-pc-windows-msvc\README.html
)

echo ----------------------------------------------------------------
echo Create bundles...
echo ----------------------------------------------------------------
pushd build
for /D %%t in (*) do (
	"%INFO_ZIP%" -r -9 %%~nxt.zip %%~nxt
	attrib +R %%~nxt.zip
)
popd

echo.
echo Completed.
echo.

pause

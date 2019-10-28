@ECHO off

:: Test for the binaries
IF NOT EXIST simsapa_dictionary.exe Goto:Error_Missing_Programs
IF NOT EXIST kindlegen.exe Goto:Error_Missing_Programs

SET SRC_NAME=%~n1
SET DEST_FILE=%SRC_NAME%.mobi

:: Generate the MOBI
simsapa_dictionary.exe markdown_to_ebook ^
    --source_path "%~1" ^
    --dict_label "" ^
    --ebook_format mobi ^
    --output_path "%DEST_FILE%" ^
    --mobi_compression 0 ^
    --kindlegen_path kindlegen.exe

Exit /b

:: Print errors

:Error_Missing_Programs
Color 0C & echo(
ECHO Usage: simsapa_dictionary.exe and kindlegen.exe must be present in the same folder as this batch script
ECHO (Waiting 5 seconds before closing...)
Timeout /T 5 /NoBreak >nul
Exit /b

:: End with a blank line.


@ECHO off

:: Test for the binaries
IF NOT EXIST simsapa_dictionary.exe Goto:Error_Missing_Programs
IF NOT EXIST kindlegen.exe Goto:Error_Missing_Programs

SET SRC_NAME=%~n1
SET DEST_FILE=%SRC_NAME%.epub

:: Generate the EPUB
simsapa_dictionary.exe markdown_to_ebook ^
    --markdown_path "%~1" ^
    --dict_label "" ^
    --ebook_format epub ^
    --output_path "%DEST_FILE%"

Exit /b

:: Print errors

:Error_Missing_Programs
Color 0C & echo(
ECHO Usage: simsapa_dictionary.exe and kindlegen.exe must be present in the same folder as this batch script
ECHO (Waiting 5 seconds before closing...)
Timeout /T 5 /NoBreak >nul
Exit /b

:: End with a blank line.


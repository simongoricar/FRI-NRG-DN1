function runReleaseBuild {
    param (
        [string[]]
        $arguments
    )

    $cargoArguments = @("run", "--release", "--");
    $appendedArguments = @("--export-screenshot-and-exit");
    $fullArguments = $($cargoArguments; $arguments; $appendedArguments);

    Write-Host -ForegroundColor Yellow "Running cargo $fullArguments";

    Start-Process `
        -FilePath "cargo" `
        -ArgumentList $fullArguments `
        -WorkingDirectory $PSScriptRoot `
        -NoNewWindow `
        -Wait;
}

$ErrorActionPreference = 'Stop';

runReleaseBuild @("--input-file-path", "./data/input-files/nike.splat", "--camera-position", "(2.1,-0.06,-0.04)")
runReleaseBuild @("--input-file-path", "./data/input-files/nike.splat", "--camera-position", "(-2.2,3.87,-1.34)")

runReleaseBuild @("--input-file-path", "./data/input-files/plush.splat", "--camera-position", "(0.41,-0.41,-0.32)")
runReleaseBuild @("--input-file-path", "./data/input-files/plush.splat", "--camera-position", "(0.19,2.71,-1.45)")

runReleaseBuild @("--input-file-path", "./data/input-files/train.splat", "--camera-position", "(-0.7,-4.1,8.8)")
runReleaseBuild @("--input-file-path", "./data/input-files/train.splat", "--camera-position", "(9,-5.6,0.9)")

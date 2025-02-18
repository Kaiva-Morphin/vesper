# Define the path to your .env file
$envFilePath = ".\.env"

# Check if the file exists
if (Test-Path $envFilePath) {
    # Read each line from the .env file
    Get-Content $envFilePath | ForEach-Object {
        # Skip empty lines and comments
        if ($_ -and $_ -notmatch "^\s*#") {
            # Split each line into key and value
            $key, $value = $_ -split "=", 2
            # Trim whitespace and set the environment variable
            [System.Environment]::SetEnvironmentVariable($key.Trim(), $value.Trim(), "Process")
        }
    }
    Write-Host "Environment variables loaded successfully from .env file."
} else {
    Write-Error "The .env file was not found at path: $envFilePath"
}

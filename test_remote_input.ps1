$uri = "wss://127.0.0.1:8080/"
$messages = @(
'{"action":"Mouse","command":"Click","click":{"button":"Left","action":"Click"},"move_direction":{"x":0,"y":0}}',
'{"action":"Mouse","command":"Click","click":{"button":"Right","action":"Click"},"move_direction":{"x":0,"y":0}}',    '{"action":"Mouse","command":"Scroll","scroll":{"direction":"Y","delta":-120},"move_direction":{"x":0,"y":0}}',
'{"action":"Mouse","command":"Move","move_direction":{"x":50,"y":0}}',
'{"action":"Mouse","command":"Move","move_direction":{"x":0,"y":50}}'
)
foreach ($msg in $messages) {
    echo $msg | c:\websocat\websocat.exe -n --insecure $uri
}

Write-Host "Test script completed."
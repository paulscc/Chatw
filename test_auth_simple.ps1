Write-Host "Prueba simple de auth de Supabase..." -ForegroundColor Green

# Configuracion
$url = "https://bsmafajdvzbpxtepxysz.supabase.co"
$anonKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzbWFmYWpkdnpicHh0ZXB4eXN6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3Nzc0NTg2MjgsImV4cCI6MjA5MzAzNDYyOH0.qI-E3efN75xEX3fdhvGoi84k0_aOOo2u9hMnxe2lbJM"

$headers = @{
    "apikey" = $anonKey
    "Authorization" = "Bearer $anonKey"
    "Content-Type" = "application/json"
}

# Probar settings
try {
    $response = Invoke-RestMethod -Uri "$url/auth/v1/settings" -Method Get -Headers $headers
    Write-Host "OK: Auth settings accesible" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Auth settings - $($_.Exception.Message)" -ForegroundColor Red
}

# Probar signup con formato correcto
try {
    $testEmail = "test$(Get-Random -Maximum 9999)@example.com"
    $body = @{
        email = $testEmail
        password = "TestPassword123!"
        data = @{
            display_name = "Test User"
        }
    } | ConvertTo-Json -Depth 3
    
    $response = Invoke-RestMethod -Uri "$url/auth/v1/signup" -Method Post -Headers $headers -Body $body
    Write-Host "OK: Signup funciona" -ForegroundColor Green
    Write-Host "Email: $testEmail" -ForegroundColor Cyan
    Write-Host "User ID: $($response.user.id)" -ForegroundColor Cyan
    
    # Guardar token para pruebas siguientes
    $token = $response.access_token
    $userId = $response.user.id
    
} catch {
    $errorResponse = $_.Exception.Response.GetResponseStream()
    $reader = New-Object System.IO.StreamReader($errorResponse)
    $reader.BaseStream.Position = 0
    $errorBody = $reader.ReadToEnd()
    Write-Host "ERROR: Signup - $errorBody" -ForegroundColor Red
}

# Probar obtener usuario si tenemos token
if ($token) {
    try {
        $userHeaders = @{
            "apikey" = $anonKey
            "Authorization" = "Bearer $token"
        }
        
        $response = Invoke-RestMethod -Uri "$url/auth/v1/user" -Method Get -Headers $userHeaders
        Write-Host "OK: Get user funciona" -ForegroundColor Green
        Write-Host "Usuario: $($response.email)" -ForegroundColor Cyan
    } catch {
        Write-Host "ERROR: Get user - $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host "Prueba completada" -ForegroundColor Green

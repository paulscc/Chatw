Write-Host "Prueba real de auth de Supabase..." -ForegroundColor Green

# Configuracion
$url = "https://bsmafajdvzbpxtepxysz.supabase.co"
$anonKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzbWFmYWpkdnpicHh0ZXB4eXN6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3Nzc0NTg2MjgsImV4cCI6MjA5MzAzNDYyOH0.qI-E3efN75xEX3fdhvGoi84k0_aOOo2u9hMnxe2lbJM"

$headers = @{
    "apikey" = $anonKey
    "Authorization" = "Bearer $anonKey"
    "Content-Type" = "application/json"
}

# Probar signup con email real
try {
    $testEmail = "testuser$(Get-Random -Maximum 9999)@gmail.com"
    $body = @{
        email = $testEmail
        password = "TestPassword123!"
        options = @{
            data = @{
                display_name = "Test User"
            }
        }
    } | ConvertTo-Json -Depth 4
    
    Write-Host "Intentando signup con: $testEmail" -ForegroundColor Yellow
    $response = Invoke-RestMethod -Uri "$url/auth/v1/signup" -Method Post -Headers $headers -Body $body
    Write-Host "OK: Signup exitoso" -ForegroundColor Green
    Write-Host "User ID: $($response.user.id)" -ForegroundColor Cyan
    
    $token = $response.access_token
    
} catch {
    $errorResponse = $_.Exception.Response.GetResponseStream()
    $reader = New-Object System.IO.StreamReader($errorResponse)
    $reader.BaseStream.Position = 0
    $errorBody = $reader.ReadToEnd()
    Write-Host "ERROR: Signup - $errorBody" -ForegroundColor Red
}

# Probar signin si signup funciono
if ($token) {
    try {
        $body = @{
            email = $testEmail
            password = "TestPassword123!"
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri "$url/auth/v1/token?grant_type=password" -Method Post -Headers $headers -Body $body
        Write-Host "OK: Signin exitoso" -ForegroundColor Green
        Write-Host "Token: $($response.access_token.Substring(0, 20))..." -ForegroundColor Cyan
        
        # Probar obtener usuario
        $userHeaders = @{
            "apikey" = $anonKey
            "Authorization" = "Bearer $($response.access_token)"
        }
        
        $userResponse = Invoke-RestMethod -Uri "$url/auth/v1/user" -Method Get -Headers $userHeaders
        Write-Host "OK: Get user exitoso" -ForegroundColor Green
        Write-Host "Usuario: $($userResponse.email)" -ForegroundColor Cyan
        Write-Host "Creado: $($userResponse.created_at)" -ForegroundColor Cyan
        
    } catch {
        Write-Host "ERROR: Signin - $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Probar conexion a base de datos
try {
    $serviceKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzbWFmYWpkdnpicHh0ZXB4eXN6Iiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc3NzQ1ODYyOCwiZXhwIjoyMDkzMDM0NjI4fQ.hGsfYrvF__FtSAfHMh1e0B_jgAOrYR40TBsoJoMUgzo"
    
    $dbHeaders = @{
        "apikey" = $serviceKey
        "Authorization" = "Bearer $serviceKey"
        "Content-Type" = "application/json"
    }
    
    # Listar tablas
    $response = Invoke-RestMethod -Uri "$url/rest/v1/" -Method Get -Headers $dbHeaders
    Write-Host "OK: Conexion a base de datos" -ForegroundColor Green
    
} catch {
    Write-Host "ERROR: Base de datos - $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Prueba completada" -ForegroundColor Green

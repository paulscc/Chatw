Write-Host "Probando endpoints de auth de Supabase..." -ForegroundColor Green

# Cargar configuracion
$envContent = Get-Content ".env"
$url = ($envContent | Select-String "SUPABASE_URL").ToString().Split("=")[1]
$anonKey = ($envContent | Select-String "SUPABASE_ANON_KEY").ToString().Split("=")[1]
$serviceKey = ($envContent | Select-String "SUPABASE_SERVICE_ROLE_KEY").ToString().Split("=")[1]

Write-Host "URL: $url" -ForegroundColor Yellow

# Headers para auth
$headers = @{
    "apikey" = $anonKey
    "Authorization" = "Bearer $anonKey"
    "Content-Type" = "application/json"
}

# Probar endpoint de settings
try {
    Write-Host "Probando auth/v1/settings..." -ForegroundColor Yellow
    $response = Invoke-RestMethod -Uri "$url/auth/v1/settings" -Method Get -Headers $headers
    Write-Host "OK: Settings endpoint accesible" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Settings endpoint - $($_.Exception.Message)" -ForegroundColor Red
}

# Probar endpoint de signup (con datos de prueba)
try {
    Write-Host "Probando auth/v1/signup..." -ForegroundColor Yellow
    $testEmail = "test$(Get-Random -Maximum 9999)@example.com"
    $testPassword = "TestPassword123!"
    
    $body = @{
        email = $testEmail
        password = $testPassword
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$url/auth/v1/signup" -Method Post -Headers $headers -Body $body
    Write-Host "OK: Signup endpoint funciona" -ForegroundColor Green
    Write-Host "Usuario creado: $testEmail" -ForegroundColor Cyan
} catch {
    Write-Host "ERROR: Signup endpoint - $($_.Exception.Message)" -ForegroundColor Red
}

# Probar endpoint de signin (con el usuario creado)
try {
    Write-Host "Probando auth/v1/token..." -ForegroundColor Yellow
    
    $body = @{
        email = $testEmail
        password = $testPassword
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$url/auth/v1/token?grant_type=password" -Method Post -Headers $headers -Body $body
    Write-Host "OK: Signin endpoint funciona" -ForegroundColor Green
    Write-Host "Token obtenido: $($response.access_token.Substring(0, 20))..." -ForegroundColor Cyan
    
    # Probar obtener usuario con token
    $userHeaders = @{
        "apikey" = $anonKey
        "Authorization" = "Bearer $($response.access_token)"
    }
    
    $userResponse = Invoke-RestMethod -Uri "$url/auth/v1/user" -Method Get -Headers $userHeaders
    Write-Host "OK: User endpoint funciona" -ForegroundColor Green
    Write-Host "Usuario: $($userResponse.email)" -ForegroundColor Cyan
    
} catch {
    Write-Host "ERROR: Signin endpoint - $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Prueba de endpoints completada" -ForegroundColor Green

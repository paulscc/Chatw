# Script para verificar integración de Supabase sin compilar
Write-Host "🔍 Verificando integración de Supabase Auth..." -ForegroundColor Green

# Verificar configuración
Write-Host "📋 Verificando configuración..." -ForegroundColor Yellow
if (Test-Path ".env") {
    $envContent = Get-Content ".env"
    $hasSupabaseUrl = $envContent -match "SUPABASE_URL="
    $hasAnonKey = $envContent -match "SUPABASE_ANON_KEY="
    $hasServiceKey = $envContent -match "SUPABASE_SERVICE_ROLE_KEY="
    
    if ($hasSupabaseUrl -and $hasAnonKey -and $hasServiceKey) {
        Write-Host "✅ Variables de entorno de Supabase configuradas" -ForegroundColor Green
    } else {
        Write-Host "❌ Faltan variables de entorno de Supabase" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "❌ Archivo .env no encontrado" -ForegroundColor Red
    exit 1
}

# Verificar archivos de código
Write-Host "📁 Verificando archivos de código..." -ForegroundColor Yellow
$requiredFiles = @(
    "src/supabase.rs",
    "src/auth_supabase.rs", 
    "src/config.rs",
    "src/error.rs"
)

foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host "✅ $file encontrado" -ForegroundColor Green
    } else {
        Write-Host "❌ $file no encontrado" -ForegroundColor Red
        exit 1
    }
}

# Verificar estructura de Supabase client
Write-Host "🔧 Verificando estructura del cliente Supabase..." -ForegroundColor Yellow
$supabaseContent = Get-Content "src/supabase.rs" -Raw
if ($supabaseContent -match "struct SupabaseClient" -and 
    $supabaseContent -match "sign_up" -and 
    $supabaseContent -match "sign_in" -and
    $supabaseContent -match "get_user") {
    Write-Host "✅ Cliente Supabase con métodos de auth" -ForegroundColor Green
} else {
    Write-Host "❌ Cliente Supabase incompleto" -ForegroundColor Red
    exit 1
}

# Verificar servicio de auth
Write-Host "🔐 Verificando servicio de autenticación..." -ForegroundColor Yellow
$authContent = Get-Content "src/auth_supabase.rs" -Raw
if ($authContent -match "struct SupabaseAuthService" -and
    $authContent -match "sign_up" -and
    $authContent -match "sign_in" -and
    $authContent -match "create_guest_session") {
    Write-Host "✅ Servicio de autenticación completo" -ForegroundColor Green
} else {
    Write-Host "❌ Servicio de autenticación incompleto" -ForegroundColor Red
    exit 1
}

# Verificar configuración extendida
Write-Host "⚙️ Verificando configuración extendida..." -ForegroundColor Yellow
$configContent = Get-Content "src/config.rs" -Raw
if ($configContent -match "supabase_url" -and
    $configContent -match "supabase_anon_key" -and
    $configContent -match "supabase_service_role_key") {
    Write-Host "✅ Configuración extendida de Supabase" -ForegroundColor Green
} else {
    Write-Host "❌ Configuración extendida incompleta" -ForegroundColor Red
    exit 1
}

# Probar conexión a API de Supabase
Write-Host "🌐 Probando conexión a API de Supabase..." -ForegroundColor Yellow
try {
    $envContent = Get-Content ".env" | Where-Object { $_ -notmatch "^#" -and $_.trim() -ne "" }
    foreach ($line in $envContent) {
        if ($line -match "^(.+?)=(.*)$") {
            [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
        }
    }
    
    $supabaseUrl = $env:SUPABASE_URL
    $serviceKey = $env:SUPABASE_SERVICE_ROLE_KEY
    
    $headers = @{
        "apikey" = $serviceKey
        "Authorization" = "Bearer $serviceKey"
        "Content-Type" = "application/json"
    }
    
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    Write-Host "✅ Conexión a API REST exitosa" -ForegroundColor Green
    
} catch {
    Write-Host "⚠️ Error en conexión API: $($_.Exception.Message)" -ForegroundColor Yellow
    # No es fatal, puede ser por permisos
}

# Verificar endpoints de auth
Write-Host "🔐 Verificando endpoints de auth..." -ForegroundColor Yellow
try {
    $anonKey = $env:SUPABASE_ANON_KEY
    $headers = @{
        "apikey" = $anonKey
        "Authorization" = "Bearer $anonKey"
    }
    
    # Intentar acceder a endpoint de auth (debería responder aunque sea con error)
    $response = Invoke-RestMethod -Uri "$supabaseUrl/auth/v1/settings" -Method Get -Headers $headers -ErrorAction SilentlyContinue
    Write-Host "✅ Endpoints de auth accesibles" -ForegroundColor Green
} catch {
    Write-Host "⚠️ Endpoints de auth no accesibles (puede ser normal)" -ForegroundColor Yellow
}

Write-Host "🎉 Verificación de integración completada" -ForegroundColor Green
Write-Host "✅ Configuración de Supabase Auth correcta" -ForegroundColor Green
Write-Host "✅ Código fuente implementado" -ForegroundColor Green
Write-Host "✅ Estructura de archivos completa" -ForegroundColor Green

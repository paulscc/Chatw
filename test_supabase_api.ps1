# Script para probar API de Supabase con la clave publishable
Write-Host "🔍 Probando API de Supabase..." -ForegroundColor Green

# Leer variables de entorno
$supabaseUrl = "https://bsmafajdvzbpxtepxysz.supabase.co"
$anonKey = "sb_publishable_C8augSiTrksX8z7ziTrvsA_ZZxzvZQU"

Write-Host "📍 URL del proyecto: $supabaseUrl" -ForegroundColor Yellow
Write-Host "🔑 Clave anónima: $anonKey" -ForegroundColor Yellow

# Probar conexión básica a la API REST
try {
    Write-Host "🔌 Probando conexión a la API REST..." -ForegroundColor Yellow
    
    $headers = @{
        "apikey" = $anonKey
        "Authorization" = "Bearer $anonKey"
    }
    
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    
    Write-Host "✅ Conexión a API REST exitosa" -ForegroundColor Green
    Write-Host "📊 Respuesta: $response" -ForegroundColor Cyan
    
} catch {
    Write-Host "❌ Error en conexión API REST: $($_.Exception.Message)" -ForegroundColor Red
}

# Probar obtener información del proyecto
try {
    Write-Host "🔍 Probando obtener información del proyecto..." -ForegroundColor Yellow
    
    $headers = @{
        "apikey" = $anonKey
        "Authorization" = "Bearer $anonKey"
    }
    
    $response = Invoke-RestMethod -Uri "$supabase.co/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    
    Write-Host "✅ Información del proyecto obtenida" -ForegroundColor Green
    
} catch {
    Write-Host "❌ Error obteniendo información: $($_.Exception.Message)" -ForegroundColor Red
}

# Probar conexión a Auth
try {
    Write-Host "🔐 Probando conexión a Auth..." -ForegroundColor Yellow
    
    $headers = @{
        "apikey" = $anonKey
        "Authorization" = "Bearer $anonKey"
    }
    
    $response = Invoke-RestMethod -Uri "$supabaseUrl/auth/v1/settings" -Method Get -Headers $headers -ErrorAction Stop
    
    Write-Host "✅ Conexión a Auth exitosa" -ForegroundColor Green
    
} catch {
    Write-Host "❌ Error en conexión Auth: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "🎉 Prueba de API completada" -ForegroundColor Green

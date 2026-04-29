# Script simple para probar API de Supabase
Write-Host "Probando API de Supabase..." -ForegroundColor Green

$supabaseUrl = "https://bsmafajdvzbpxtepxysz.supabase.co"
$anonKey = "sb_publishable_C8augSiTrksX8z7ziTrvsA_ZZxzvZQU"

Write-Host "URL del proyecto: $supabaseUrl" -ForegroundColor Yellow
Write-Host "Clave anonima: $anonKey" -ForegroundColor Yellow

try {
    $headers = @{
        "apikey" = $anonKey
        "Authorization" = "Bearer $anonKey"
    }
    
    Write-Host "Probando conexion a API REST..." -ForegroundColor Yellow
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    Write-Host "Conexion API REST exitosa" -ForegroundColor Green
    
} catch {
    Write-Host "Error en conexion API REST: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Prueba de API completada" -ForegroundColor Green

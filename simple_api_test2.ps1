$supabaseUrl = "https://bsmafajdvzbpxtepxysz.supabase.co"
$serviceKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzbWFmYWpkdnpicHh0ZXB4eXN6Iiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc3NzQ1ODYyOCwiZXhwIjoyMDkzMDM0NjI4fQ.hGsfYrvF__FtSAfHMh1e0B_jgAOrYR40TBsoJoMUgzo"

Write-Host "Probando API con clave de servicio..." -ForegroundColor Green

$headers = @{
    "apikey" = $serviceKey
    "Authorization" = "Bearer $serviceKey"
    "Content-Type" = "application/json"
}

try {
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    Write-Host "Conexion API exitosa" -ForegroundColor Green
    Write-Host "Respuesta: $response" -ForegroundColor Cyan
} catch {
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Verificando tablas..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/users?select=*" -Method Get -Headers $headers -ErrorAction Stop
    Write-Host "Tabla users encontrada" -ForegroundColor Green
} catch {
    Write-Host "Error accediendo a users: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Prueba completada" -ForegroundColor Green

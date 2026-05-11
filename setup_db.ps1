Write-Host "Verificando conexión a PostgreSQL..." -ForegroundColor Green

# Intentar conectar a PostgreSQL y crear la base de datos si no existe
try {
    # Verificar si psql está disponible
    $psqlVersion = psql --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "PostgreSQL client encontrado: $psqlVersion" -ForegroundColor Cyan
        
        # Intentar conectar y crear la base de datos
        Write-Host "Intentando conectar a postgresql://postgres:admin@localhost:5432" -ForegroundColor Yellow
        
        # Verificar si la base de datos chatu existe
        $dbCheck = psql -h localhost -U postgres -p 5432 -d postgres -c "SELECT 1 FROM pg_database WHERE datname='chatu';" -t -A 2>$null
        
        if ($dbCheck -match "1") {
            Write-Host "Base de datos 'chatu' ya existe" -ForegroundColor Green
        } else {
            Write-Host "Creando base de datos 'chatu'..." -ForegroundColor Yellow
            psql -h localhost -U postgres -p 5432 -d postgres -c "CREATE DATABASE chatu;" 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Base de datos 'chatu' creada exitosamente" -ForegroundColor Green
            } else {
                Write-Host "Error al crear base de datos" -ForegroundColor Red
            }
        }
        
        # Probar conexión a la base de datos chatu
        Write-Host "Probando conexión a la base de datos 'chatu'..." -ForegroundColor Yellow
        $testConnection = psql -h localhost -U postgres -p 5432 -d chatu -c "SELECT version();" -t -A 2>$null
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Conexión a PostgreSQL exitosa" -ForegroundColor Green
            Write-Host "Versión: $testConnection" -ForegroundColor Cyan
        } else {
            Write-Host "Error al conectar a la base de datos 'chatu'" -ForegroundColor Red
        }
        
    } else {
        Write-Host "PostgreSQL client (psql) no encontrado" -ForegroundColor Red
        Write-Host "Instala PostgreSQL o usa Docker" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Verificación completada" -ForegroundColor Green

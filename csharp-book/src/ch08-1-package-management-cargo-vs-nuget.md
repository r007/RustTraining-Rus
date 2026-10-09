## Управление пакетами: Cargo против NuGet

> **Что вы узнаете:** `Cargo.toml` против `.csproj`, спецификаторы версий, `Cargo.lock`,
> флаги функций для условной компиляции, а также распространённые команды Cargo и их аналоги в NuGet/dotnet.
>
> **Сложность:** 🟢 Начальный

### Объявление зависимостей

#### Зависимости NuGet в C#
```xml
<!-- MyApp.csproj -->
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  
  <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
  <PackageReference Include="Serilog" Version="3.0.1" />
  <PackageReference Include="Microsoft.AspNetCore.App" />
  
  <ProjectReference Include="../MyLibrary/MyLibrary.csproj" />
</Project>
```

#### Зависимости Cargo в Rust
```toml
# Cargo.toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde_json = "1.0"               # Из crates.io (как NuGet)
serde = { version = "1.0", features = ["derive"] }  # С функциями
log = "0.4"
tokio = { version = "1.0", features = ["full"] }

# Локальные зависимости (как ProjectReference)
my_library = { path = "../my_library" }

# Зависимости из git
my_git_crate = { git = "https://github.com/user/repo" }

# Зависимости для разработки (как тестовые пакеты)
[dev-dependencies]
criterion = "0.5"               # Бенчмарки
proptest = "1.0"               # Property-тестирование
```

### Управление версиями

#### Версионирование пакетов в C#
```xml
<!-- Централизованное управление пакетами (Directory.Packages.props) -->
<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
  </PropertyGroup>
  
  <PackageVersion Include="Newtonsoft.Json" Version="13.0.3" />
  <PackageVersion Include="Serilog" Version="3.0.1" />
</Project>

<!-- packages.lock.json для воспроизводимых сборок -->
```

#### Управление версиями в Rust
```toml
# Cargo.toml — семантическое версионирование
[dependencies]
serde = "1.0"        # Совместимо с 1.x.x (>=1.0.0, <2.0.0)
log = "0.4.17"       # Совместимо с 0.4.x (>=0.4.17, <0.5.0)
regex = "=1.5.4"     # Точная версия
chrono = "^0.4"      # Требования с кареткой (по умолчанию)
uuid = "~1.3.0"      # Требования с тильдой (>=1.3.0, <1.4.0)

# Cargo.lock — точные версии для воспроизводимых сборок (генерируется автоматически)
[[package]]
name = "serde"
version = "1.0.163"
# ... точное дерево зависимостей
```

### Источники пакетов

#### Источники пакетов в C#
```xml
<!-- nuget.config -->
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
    <add key="MyCompanyFeed" value="https://pkgs.dev.azure.com/company/_packaging/feed/nuget/v3/index.json" />
  </packageSources>
</configuration>
```

#### Источники пакетов в Rust
```toml
# .cargo/config.toml
[source.crates-io]
replace-with = "my-awesome-registry"

[source.my-awesome-registry]
registry = "https://my-intranet:8080/index"

# Альтернативные реестры
[registries]
my-registry = { index = "https://my-intranet:8080/index" }

# В Cargo.toml
[dependencies]
my_crate = { version = "1.0", registry = "my-registry" }
```

### Сравнение распространённых команд

| Задача | Команда C# | Команда Rust |
|------|------------|-------------|
| Восстановить пакеты | `dotnet restore` | `cargo fetch` |
| Добавить пакет | `dotnet add package Newtonsoft.Json` | `cargo add serde_json` |
| Удалить пакет | `dotnet remove package Newtonsoft.Json` | `cargo remove serde_json` |
| Обновить пакеты | `dotnet update` | `cargo update` |
| Список пакетов | `dotnet list package` | `cargo tree` |
| Аудит безопасности | `dotnet list package --vulnerable` | `cargo audit` |
| Чистая сборка | `dotnet clean` | `cargo clean` |

### Features: условная компиляция

#### Условная компиляция в C#
```csharp
#if DEBUG
    Console.WriteLine("Debug mode");
#elif RELEASE
    Console.WriteLine("Release mode");
#endif

// Функции в файле проекта
<PropertyGroup Condition="'$(Configuration)'=='Debug'">
    <DefineConstants>DEBUG;TRACE</DefineConstants>
</PropertyGroup>
```

#### Feature-флаги в Rust
```toml
# Cargo.toml
[features]
default = ["json"]              # Функции по умолчанию
json = ["serde_json"]          # Функция, включающая serde_json
xml = ["serde_xml"]            # Альтернативная сериализация
advanced = ["json", "xml"]     # Составная функция

[dependencies]
serde_json = { version = "1.0", optional = true }
serde_xml = { version = "0.4", optional = true }
```

```rust
// Условная компиляция на основе функций
#[cfg(feature = "json")]
use serde_json;

#[cfg(feature = "xml")]
use serde_xml;

pub fn serialize_data(data: &MyStruct) -> String {
    #[cfg(feature = "json")]
    return serde_json::to_string(data).unwrap();
    
    #[cfg(feature = "xml")]
    return serde_xml::to_string(data).unwrap();
    
    #[cfg(not(any(feature = "json", feature = "xml")))]
    return "No serialization feature enabled".to_string();
}
```

### Использование внешних крейтов

#### Популярные крейты для разработчиков C#

| Библиотека C# | Крейт Rust | Назначение |
|------------|------------|---------|
| System.Text.Json / Newtonsoft.Json | `serde_json` | Сериализация JSON |
| HttpClient | `reqwest` | HTTP-клиент |
| Entity Framework | `diesel` / `sqlx` | ORM / инструментарий SQL |
| NLog/Serilog | `log` + `env_logger` | Логирование |
| xUnit/NUnit | Встроенный `#[test]` | Модульное тестирование |
| Moq | `mockall` | Моки |
| Flurl | `url` | Работа с URL |
| Polly | `tower` | Паттерны отказоустойчивости |

#### Пример: миграция HTTP-клиента
```csharp
// Использование HttpClient в C#
public class ApiClient
{
    private readonly HttpClient _httpClient;
    
    public async Task<User> GetUserAsync(int id)
    {
        var response = await _httpClient.GetAsync($"/users/{id}");
        var json = await response.Content.ReadAsStringAsync();
        return System.Text.Json.JsonSerializer.Deserialize<User>(json);
    }
}
```

```rust
// Использование reqwest в Rust
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    id: u32,
    name: String,
}

struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    async fn get_user(&self, id: u32) -> Result<User, reqwest::Error> {
        let user = self.client
            .get(&format!("https://api.example.com/users/{}", id))
            .send()
            .await?
            .json::<User>()
            .await?;
        
        Ok(user)
    }
}
```

***



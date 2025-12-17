use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Interrupted(String),
    Validation(String),
    Docker(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "Erro de I/O: {}", err),
            AppError::Interrupted(msg) => write!(f, "Execução Interrompida: {}", msg),
            AppError::Validation(msg) => write!(f, "Erro de validação: {}", msg),
            AppError::Docker(msg) => write!(f, "Erro no Docker: {}", msg),
        }
    }
}

impl Error for AppError {}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::Io(err)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> AppError {
        AppError::Validation(format!("Falha ao converter número: {}", err))
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> AppError {
        AppError::Validation(format!("Variável de ambiente não encontrada: {}", err))
    }
}

const ENV_FILE: &str = ".env";
const EXAMPLE_ENV_FILE: &str = "env.example";
const DEFAULT_CONTAINER_NAME: &str = "dev_container";
const DEFAULT_SERVER_PORT: u16 = 8000;
const DEFAULT_DB_PORT: u16 = 3306;
const DEFAULT_DB_ROOT_PASSWORD: &str = "password";
const VHOSTS_DIR: &str = "docker/apache/vhosts";
const DEFAULT_LARAVEL_VERSION: u8 = 12;
const MINIMAL_LARAVEL_VERSION: u8 = 10;

#[derive(Debug)]
struct AppConfig {
    php_container_name: String,
    node_container_name: String,
    db_root_password: String,
    server_port: u16,
    db_port: u16,
}

#[derive(Debug)]
struct ProjectInput {
    project_name: String,
    project_host: String,
    project_path: String,
    laravel_version: String,
}

fn run() -> Result<(), AppError> {
    println!("--- Dev Container Laravel Maker ---");

    let env_path_option = find_env_path(ENV_FILE);
    let example_env_path_option = find_env_path(EXAMPLE_ENV_FILE);

    let env_path = ensure_env_file_exists(env_path_option, example_env_path_option)?;

    dotenv::from_path(&env_path).ok();

    let config = get_app_config()?;
    let input = get_user_input()?;

    execute_laravel_creation(&input, &config)?;

    configure_and_initialize_laravel(&input, &config)?;

    create_vhost_file(&input)?;

    update_etc_hosts(&input)?;

    restart_apache_container()?;

    println!("\n---");
    println!(
        "Novo projeto Laravel '{}' criado com sucesso!",
        input.project_name
    );
    println!(
        "Domínio de acesso: http://{}:{}",
        input.project_host, config.server_port
    );
    println!("---");
    println!("O projeto está pronto. Você já pode acessá-lo pelo navegador.");

    Ok(())
}

fn find_env_path(filename: &str) -> Option<PathBuf> {
    let path_dot = PathBuf::from(filename);
    if path_dot.exists() {
        return Some(path_dot);
    }

    let path_dot_dot = PathBuf::from("..").join(filename);
    if path_dot_dot.exists() {
        return Some(path_dot_dot);
    }

    None
}

fn ensure_env_file_exists(
    env_path_option: Option<PathBuf>,
    example_env_path_option: Option<PathBuf>,
) -> Result<PathBuf, AppError> {
    if let Some(env_path) = env_path_option {
        println!("Arquivo .env encontrado.");
        return Ok(env_path);
    }

    println!("Arquivo .env não encontrado. Tentando criar a partir do env.example... ");

    let example_env_path = match example_env_path_option {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Nem o .env, nem o . env.example foram encontrados. Verifique a estrutura do projeto."),
            ).into());
        }
    };

    let env_path = example_env_path.with_file_name(".env");

    match fs::copy(&example_env_path, &env_path) {
        Ok(_) => {
            println!(
                "Copiado {} para {} ",
                example_env_path.display(),
                env_path.display()
            );

            loop {
                println!("\n--- Configuração Inicial ---");
                println!("O arquivo de configuração .env foi criado com as variáveis padrão.");
                print!("Deseja prosseguir com a configuração padrão do .env ? (Y/n, ENTER=Y): ");
                io::stdout().flush()?;

                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
                let choice = buffer.trim().to_lowercase();

                if choice.is_empty() || choice == "y" {
                    println!("Continuando com as configurações padrão do .env.");
                    return Ok(env_path);
                } else if choice == "n" {
                    println!(
                        "\nProcesso interrompido. Edite o arquivo .env e execute o programa novamente."
                    );
                    println!("Pressione [Enter] para sair...");
                    io::stdout().flush()?;
                    let mut exit_buffer = String::new();
                    io::stdin().read_line(&mut exit_buffer)?;

                    return Err(AppError::Interrupted(
                        "O usuário optou por configurar o .env manualmente.".to_string(),
                    ));
                } else {
                    println!("Escolha inválida");
                }
            }
        }
        Err(e) => {
            return Err(e.into());
        }
    }
}

fn get_app_config() -> Result<AppConfig, AppError> {
    println!("Carregando configurações do .env...");

    let container_name = match env::var("CONTAINER_NAME") {
        Ok(name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => {
            println!(
                "CONTAINER_NAME não encontrado ou vazio. Usando default: '{}'",
                DEFAULT_CONTAINER_NAME
            );
            DEFAULT_CONTAINER_NAME.to_string()
        }
    };

    let server_port = match env::var("SERVER_PORT") {
        Ok(port_str) => match port_str.trim().parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                println!(
                    "SERVER_PORT ('{}') inválido. Usando default: {}",
                    port_str.trim(),
                    DEFAULT_SERVER_PORT
                );
                DEFAULT_SERVER_PORT
            }
        },
        Err(_) => {
            println!(
                "SERVER_PORT não encontrado. Usando default: {}",
                DEFAULT_SERVER_PORT
            );
            DEFAULT_SERVER_PORT
        }
    };

    let db_port = match env::var("DB_PORT") {
        Ok(port_str) => match port_str.trim().parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                println!(
                    "DB_PORT ('{}') inválido. Usando default: {}",
                    port_str.trim(),
                    DEFAULT_DB_PORT
                );
                DEFAULT_DB_PORT
            }
        },
        Err(_) => {
            println!(
                "DB_PORT não encontrado. Usando default: {}",
                DEFAULT_DB_PORT
            );
            DEFAULT_DB_PORT
        }
    };

    let php_container_name = format!("{}_php", container_name);
    let node_container_name = format!("{}_node", container_name);

    let db_root_password = match env::var("DB_ROOT_PASSWORD") {
        Ok(password) if !password.trim().is_empty() => password.trim().to_string(),
        _ => {
            println!(
                "MYSQL_ROOT_PASSWORD não encontrada ou vazia. Usando default: '{}'",
                DEFAULT_DB_ROOT_PASSWORD
            );
            DEFAULT_DB_ROOT_PASSWORD.to_string()
        }
    };

    println!(
        "Configurações base carregadas (Contêiner PHP: {}, Porta Apache: {})",
        php_container_name, server_port
    );

    Ok(AppConfig {
        php_container_name,
        node_container_name,
        db_root_password,
        server_port,
        db_port,
    })
}

fn get_user_input() -> Result<ProjectInput, AppError> {
    let project_name = 'project_loop: loop {
        print!("Digite o NOME do novo projeto (ex: example-app): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let raw_name = input.trim().to_lowercase();

        if raw_name.is_empty() {
            eprintln!("O nome do projeto não pode ser vazio.");
            continue;
        }
        let name = format_to_kebab_case(&raw_name);

        if name.is_empty() {
            eprintln!(
                "A entrada original resultou em um nome vazio após a formatação. Tente novamente."
            );
            continue;
        }

        if name != raw_name {
            println!(
                "Formatado: '{}' alterado para '{}' (kebab-case).",
                raw_name, name
            );
        }

        let project_path_check = PathBuf::from(format!("../src/{}", name));
        if project_path_check.exists() {
            eprintln!("ERRO DE VALIDAÇÃO: O diretório ../src/{} já existe.", name);

            loop {
                print!("Deseja tentar outro nome de projeto? (Y/n, ENTER=Y): ");
                io::stdout().flush()?;

                let mut decision = String::new();
                io::stdin().read_line(&mut decision)?;
                let choice = decision.trim().to_lowercase();

                if choice.is_empty() || choice == "y" {
                    continue 'project_loop;
                } else if choice == "n" {
                    return Err(AppError::Interrupted(
                        "O usuário optou por encerrar a aplicação.".to_string(),
                    ));
                } else {
                    eprintln!("Escolha inválida ('{}'). Digite 'Y' ou 'n'.", choice);
                }
                continue;
            }
        }
        break name;
    };

    let laravel_version = loop {
        println!("---");
        println!(
            "Versões de Laravel Comuns: {} (LTS), 11 (Mínimo aceito: {})",
            DEFAULT_LARAVEL_VERSION, MINIMAL_LARAVEL_VERSION
        );
        print!(
            "Digite a versão do Laravel (ex: {ver}, ENTER={ver}, Min={min}): ",
            ver = DEFAULT_LARAVEL_VERSION,
            min = MINIMAL_LARAVEL_VERSION
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let version_str = input.trim().to_string();

        if version_str.is_empty() {
            let default_version = DEFAULT_LARAVEL_VERSION.to_string();
            println!("Usando default: {}.", default_version);
            break default_version;
        }

        match version_str.parse::<u8>() {
            Ok(version_num) => {
                if version_num >= MINIMAL_LARAVEL_VERSION {
                    break version_num.to_string();
                } else {
                    eprintln!(
                        "ERRO: A versão informada ({}) é inválida. O campo é obrigatório e a versão mínima aceita é {}.",
                        version_num, MINIMAL_LARAVEL_VERSION
                    );
                    continue;
                }
            }
            Err(_) => {
                eprintln!(
                    "ERRO: O dado informado ('{}') é inválido. Por favor, digite apenas o número inteiro da versão (ex: {ver}, ENTER={ver}).",
                    version_str,
                    ver = DEFAULT_LARAVEL_VERSION
                );
                continue;
            }
        }
    };

    let project_host = format!("{}.test", project_name);
    let project_path = format!("../src/{}", project_name);

    println!("---");
    println!(
        "Entradas válidas: Projeto='{}', Host='{}', Versão='{}'",
        project_name, project_host, laravel_version
    );
    println!("---");

    Ok(ProjectInput {
        project_name,
        project_host,
        project_path,
        laravel_version,
    })
}

fn format_to_kebab_case(input: &str) -> String {
    let lower = input.to_lowercase();
    let mut result = lower
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>();

    result = result.split_whitespace().collect::<Vec<&str>>().join("-");

    while result.contains("--") {
        result = result.replace("--", "-");
    }

    result.trim_matches('-').to_string()
}

fn find_project_root() -> Option<PathBuf> {
    let path_dot = PathBuf::from("./docker");
    if path_dot.exists() && path_dot.is_dir() {
        return Some(PathBuf::from("."));
    }

    let path_dot_dot = PathBuf::from("../docker");
    if path_dot_dot.exists() && path_dot_dot.is_dir() {
        return Some(PathBuf::from(".."));
    }
    None
}

fn create_vhost_file(input: &ProjectInput) -> Result<(), AppError> {
    println!("Criando arquivo de configuração Vhost...");

    let project_root = find_project_root().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Não foi possível determinar o diretório raiz do projeto {}.",
                input.project_name
            ),
        )
    })?;

    let vhosts_dir =project_root.join(VHOSTS_DIR);
    let vhost_filename = format!("{}.conf", input.project_host);
    let vhost_path = vhosts_dir.join(&vhost_filename);

    let vhost_content = format!(
        r#"<VirtualHost *:80>
    # Nome do host que será usado (ex: minha-app.test)
    ServerName {}

    # Diretório raiz do projeto Laravel (montado em /var/www/html/)
    DocumentRoot /var/www/html/{}/public

    <Directory /var/www/html/{}/public>
        AllowOverride All
         Require all granted
        DirectoryIndex index.php index.html
    </Directory>

    <FilesMatch \.php$>
        SetHandler "proxy:fcgi://php:9000"
    </FilesMatch>
</VirtualHost>"#,
        input.project_host, input.project_name, input.project_name
    );
    fs::write(&vhost_path, vhost_content)?;

    println!("Vhost criado com sucesso: {}", vhost_path.display());

    Ok(())
}

fn execute_laravel_creation(input: &ProjectInput, config: &AppConfig) -> Result<(), AppError> {
    println!(">> Instalando Laravel ({})", input.laravel_version);

    let check_container_is_running = |name: &str| -> Result<bool, io::Error> {
        let output = Command::new("docker")
            .arg("ps")
            .arg("-q")
            .arg("-f")
            .arg(format!("name={}", name))
            .output()?;

        let status = String::from_utf8_lossy(&output.stdout);
        Ok(!status.trim().is_empty())
    };

    match check_container_is_running(&config.php_container_name) {
        Ok(true) => {
            println!("Contêiner PHP ativo.");
        }
        _ => {
            println!(
                "Contêiner PHP '{}' não está ativo. Iniciando o ambiente Docker Compose...",
                config.php_container_name
            );
            let up_status = Command::new("docker")
                .arg("compose")
                .arg("up")
                .arg("-d")
                .status()
                .map_err(|e| {
                    AppError::Docker(format!("Falha ao executar 'docker compose up -d': {}", e))
                })?;

            if !up_status.success() {
                return Err(AppError::Docker(
                    "Falha ao iniciar o ambiente Docker Compose. Verifique as configurações."
                        .to_string(),
                ));
            }

            let max_attempts = 3;
            let wait_time = std::time::Duration::from_secs(3);

            for attempt in 1..=max_attempts {
                println!(
                    "Aguardando inicialização do contêiner PHP (Tentativa {} de {})...",
                    attempt, max_attempts
                );
                io::stdout().flush()?;

                std::thread::sleep(wait_time);

                match check_container_is_running(&config.php_container_name) {
                    Ok(true) => {
                        println!("\rContêiner PHP ativo e pronto."); // Limpa a linha
                        break;
                    }
                    Ok(false) if attempt == max_attempts => {
                        return Err(AppError::Docker(format!(
                            "O contêiner PHP '{}' falhou ao iniciar após {} tentativas.",
                            config.php_container_name, max_attempts
                        )));
                    }
                    Err(e) => {
                        return Err(AppError::Docker(format!(
                            "Falha ao verificar o status do contêiner: {}",
                            e
                        )));
                    }
                    _ => continue,
                }
            }
        }
    }

    let status = Command::new("docker")
        .arg("exec")
        .arg("-it")
        .arg(&config.php_container_name)
        .arg("composer")
        .arg("create-project")
        .arg("laravel/laravel")
        .arg(&input.project_name)
        .arg(&input.laravel_version)
        .status()
        .map_err(|e| {
            AppError::Docker(format!("Falha ao executar 'docker exec composer': {}", e))
        })?;

    if !status.success() {
        return Err(AppError::Docker(
            "Composer falhou ao criar o projeto. Verifique logs do contêiner.".to_string(),
        ));
    }

    println!(
        "Projeto Laravel '{}' criado com sucesso em {}",
        input.project_name, input.project_path
    );
    Ok(())
}

fn restart_apache_container() -> Result<(), AppError> {
    println!("---");
    println!("Reiniciando o contêiner Apache para carregar o novo Vhost...");

    let status = Command::new("docker")
        .arg("compose")
        .arg("restart")
        .arg("apache")
        .status()
        .map_err(|e| {
            AppError::Docker(format!("Falha ao executar 'docker compose restart': {}", e))
        })?;

    if status.success() {
        std::thread::sleep(std::time::Duration::from_secs(1));

        println!("\rContêiner Apache reiniciado com sucesso.");
        io::stdout().flush()?;

        Ok(())
    } else {
        return Err(AppError::Docker(format!(
            "Falha ao reiniciar o contêiner Apache. Verifique se o serviço 'apache' está correto no docker-compose.yml. Status: {:?}",
            status
        )));
    }
}

fn update_etc_hosts(input: &ProjectInput) -> Result<(), AppError> {
    use std::process::Command;

    println!("---");
    println!(
        "O próximo passo exige permissão de administrador (sudo) para atualizar o /etc/hosts."
    );

    let host_entry = format!("127.0.0.1 {}", input.project_host);
    let hosts_file_path = "/etc/hosts";

    match fs::read_to_string(hosts_file_path) {
        Ok(content) => {
            if content.contains(&input.project_host) {
                println!(
                    "✅ Entrada de host '{}' já existe em /etc/hosts.",
                    input.project_host
                );
                return Ok(());
            }
        }
        Err(e) => {
            println!(
                "Não foi possível ler /etc/hosts para verificação: {}. Tentando escrever com sudo.",
                e
            );
        }
    }

    let command_string = format!("echo '{}' >> {}", host_entry, hosts_file_path);

    let status = Command::new("sudo")
        .arg("sh")
        .arg("-c")
        .arg(command_string)
        .status()
        .map_err(|e| AppError::Io(e.into()))?; // Trata erros de IO ao executar sudo

    if status.success() {
        println!("Host '{}' adicionado a /etc/hosts.", input.project_host);
    } else {
        return Err(AppError::Validation(format!(
            "Falha ao executar 'sudo'. Verifique se você digitou a senha corretamente. Status: {:?}",
            status
        )));
    }

    Ok(())
}

fn execute_command_in_container(container_name: &str, args: &[&str]) -> Result<(), AppError> {
    let status = Command::new("docker")
        .arg("exec")
        .arg("-it")
        .arg(container_name)
        .args(args)
        .status()
        .map_err(|e| {
            AppError::Docker(format!(
                "Falha ao executar comando no contênier '{}':{}",
                container_name, e
            ))
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Docker(format!(
            "Comando falho dentro do contêiner '{}'. Status: {:?}",
            container_name, status,
        )))
    }
}

fn configure_and_initialize_laravel(
    input: &ProjectInput,
    config: &AppConfig,
) -> Result<(), AppError> {
    println!("---");
    println!("Iniciando configurações e inicialização do projeto Laravel...");

    println!(">> Configurando arquivo .env...");
    let env_updates = vec![
        format!(
            "s/APP_URL=http:\\/\\/localhost/APP_URL=http:\\/\\/{}/",
            input.project_host
        ),
        "s/DB_CONNECTION=sqlite/DB_CONNECTION=mariadb/".to_string(),
        format!("s/# DB_PORT=3306/DB_PORT={}/", config.db_port),
        format!(
            "s/# DB_DATABASE=laravel/DB_DATABASE={}/",
            input.project_name
        ),
        "s/# DB_HOST=127.0.0.1/DB_HOST=mariadb/".to_string(),
        "s/# DB_USERNAME=root/DB_USERNAME=root/".to_string(),
        format!("s/# DB_PASSWORD=/DB_PASSWORD={}/", config.db_root_password),
    ];

    for update in env_updates {
        let command_str = format!(
            "cd /var/www/html/{} && sed -i '{}' .env",
            input.project_name, update
        );

        let args: Vec<&str> = vec!["sh", "-c", command_str.as_str()];

        let status = Command::new("docker")
            .arg("exec")
            .arg("-it")
            .arg(&config.php_container_name)
            .args(&args)
            .status()
            .map_err(|e| AppError::Docker(format!("Falha ao executar sed para .env: {}", e)))?;

        if !status.success() {
            return Err(AppError::Docker(format!(
                "Falha ao atualizar o .env com: '{}'. Status: {:?}",
                update, status
            )));
        }
    }

    println!("Arquivo .env configurado.");
    println!(">> Executando comandos Artisan (config:clear, migrate)...");

    execute_command_in_container(
        &config.php_container_name,
        &[
            "sh",
            "-c",
            &format!(
                "cd /var/www/html/{} && php artisan config:clear",
                input.project_name
            ),
        ],
    )?;
    execute_command_in_container(
        &config.php_container_name,
        &[
            "sh",
            "-c",
            &format!(
                "cd /var/www/html/{} && php artisan migrate --force",
                input.project_name
            ),
        ],
    )?;

    println!(">> Executando composer update...");
    execute_command_in_container(
        &config.php_container_name,
        &[
            "sh",
            "-c",
            &format!("cd /var/www/html/{} && composer update", input.project_name),
        ],
    )?;

    println!(">> Executando npm install...");
    execute_command_in_container(
        &config.node_container_name,
        &[
            "sh",
            "-c",
            &format!("cd /var/www/html/{} && npm install", input.project_name),
        ],
    )?;

    println!(">> Configurando vite.config.js...");

    let vite_update = "s|});$|\\tserver: {\\n\\t\\thost: '0.0.0.0'\\n\\t}\\n});|";

    let command_str = format!(
        "cd /var/www/html/{} && sed -i \"{}\" vite.config.js",
        input.project_name, vite_update
    );

    let args: Vec<&str> = vec!["sh", "-c", command_str.as_str()];

    let status = Command::new("docker")
        .arg("exec")
        .arg("-it")
        .arg(&config.php_container_name)
        .args(&args)
        .status()
        .map_err(|e| {
            AppError::Docker(format!("Falha ao executar sed para vite.config.js: {}", e))
        })?;

    if !status.success() {
        return Err(AppError::Docker(format!(
            "Falha ao atualizar o vite.config.js com: '{}'. Status: {:?}",
            vite_update, status,
        )));
    }

    println!("vite.config.js configurado com sucesso.");

    println!(
        "Projeto '{}' completamente inicializado.",
        input.project_name
    );

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {
            println!("\n Rotina concluída com sucesso.");
        }
        Err(e) => {
            eprintln!("\n Falha na execução: {}", e);
            std::process::exit(1);
        }
    }
}

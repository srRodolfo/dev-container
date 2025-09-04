# Projeto Dev-Container - Ambiente de Desenvolvimento Docker

Este repositório contém um **ambiente de desenvolvimento completo** utilizando Docker, pronto para PHP 8.0 até o 8.4 (FPM), Apache, MariaDB e Node.js.  

O ambiente foi configurado para ser usado com IDEs como PHPStorm ou VSCode, com **Composer, NPM e Xdebug integrados**.

---

### Estrutura do Projeto

- `docker/` Arquivos de configuração do Docker
- `docker/php/` Dockerfile do PHP-FPM + Node.js + Composer + Xdebug
- `docker/apache/` Dockerfile do Apache e arquivos de configuração
- `docker/.env` Configurações de ambiente (portas, usuários, senhas)
- `docker/docker-compose.yml` Orquestração dos serviços (PHP, Apache, MariaDB)
- `/` Código-fonte do projeto (montado nos containers)
- `/public` Pasta pública do projeto (raiz do servidor web, acessada pelo navegador)

```
project-root/
 ├─ public/index.php
 └─ docker/
     ├─ docker-compose.yml
     ├─ .env
     ├─ php/
     ├─ apache/
     └─ mysql/
```
---

### Pré-requisitos

- Docker e Docker Compose instalados
- IDE configurada para usar PHP dentro do container (opcional, mas recomendado)
- Sistema operacional compatível (Linux, macOS ou Windows)

---

### Configuração do Ambiente

1. Copie o arquivo `.env.example` para `.env` e ajuste as variáveis conforme necessário:

```dotenv
# Nome do container
CONTAINER_NAME=dev_container

# PHP
PUID=1000
PGID=1000
PHP_PORT=9000
PHP_VERSION=8.4

# Apache
APACHE_PORT=8080

# MariaDB
MYSQL_ROOT_PASSWORD=senha_admin
MYSQL_DATABASE=nome_banco
MYSQL_USER=nome_usuario
MYSQL_PASSWORD=senha_usuario
MYSQL_PORT=3306
```
2. Subir o ambiente com Docker Compose:

```bash
docker compose up -d --build
```
- `--build` garante que as imagens sejam construídas caso haja alterações no Dockerfile.
- O PHP-FPM estará disponível na versão definida em `PHP_VERSION` e na porta definida em `PHP_PORT`.
- O Apache estará disponível na porta definida em `APACHE_PORT` (ex: http://localhost:8080).

### Acessando o ambiente

- PHP: integrado ao container php
- Apache: `http://localhost:<APACHE_PORT>`
- MariaDB: host `mysql:<MYSQL_PORT>`, usuário `<MYSQL_USER>` e senha `<MYSQL_PASSWORD>` no arquivo `.env`
- Node.js / NPM: dentro do container PHP (node -v, npm -v)
- Composer: dentro do container PHP (composer install)

Você pode executar comandos diretamente da IDE apontando para o container PHP `<CONTAINER_NAME>_php`.

### Configuração de Xdebug

- Porta configurada: `9003`
- Host: `host.docker.internal`
- Ativado para debug remoto em IDE
- Exibição de erros do PHP está habilitada `(display_errors=On, error_reporting=E_ALL)`

### Volumes e Persistência

- Código-fonte é montado na raiz do projeto no host `/` para `/var/www/html` dentro do container
- Banco de dados MariaDB persiste em volume `db_data` para manter dados entre reinicializações

### Comandos úteis

Ver logs de containers:
```bash
docker compose logs -f
```

Acessar terminal do container PHP:
```bash
docker exec -it dev_container_php bash
```

Rodar Composer / NPM dentro do container PHP:
```bash
docker exec dev_container_php composer install
docker exec dev_container_php npm install
```

Parar o ambiente:
```bash
docker compose down
```

### Dicas

- Não é necessário instalar PHP, Composer ou Node localmente.
- Para atualizar dependências do Composer: `docker compose exec dev_container_php composer update`
- Para rodar scripts Node/NPM: `docker compose exec dev_container_php npm run <script>`

### Instalar Laravel (opcional)

Na raiz do projeto execute este comando:
```bash
rm -rf public
docker exec dev_container_php composer create-project laravel/laravel . "12.*"
```

#### Observações importantes!

O comando `rm -rf public` remove a pasta `public` criada automaticamente na raiz do projeto `/`, para evitar conflitos com o `public` do Laravel.

O ponto `.` no comando significa que o Laravel será instalado na raiz do projeto `/`.
Após a instalação, o DocumentRoot no Apache aponta para `public`.

É obrigatório informar a versão do Laravel entre aspas duplas, por exemplo:

- "12.*" → instala a versão 12
- "11.*" → instala a versão 11

### Atalhos para o Terminal (Opcional)

Adicione o seguinte ao seu `~/.bashrc` ou `~/.bash_aliases`:

```bash
# Função única para PHP/Composer/NPM/Node no container
docker_php_tools() {
  local tool="$1"   # php, composer, npm ou node
  shift
  local container=$(docker ps --format '{{.Names}}' | grep '_php$' | head -n 1)

  if [ -n "$container" ]; then
    docker exec -it "$container" "$tool" "$@"
  else
    echo "Nenhum container PHP em execução. Rodando '$tool' no host."
    command "$tool" "$@"
  fi
}

# Aliases
alias up='docker compose up -d'
alias down='docker compose down'
alias php='docker_php_tools php'
alias composer='docker_php_tools composer'
alias npm='docker_php_tools npm'
alias node='docker_php_tools node'
```
Depois, recarregue o Bash:
```bash
source ~/.bashrc
```
Como usar:
```bash
# Ver versão do PHP
php -v

# Instalar dependências do Composer
composer install

# Rodar scripts NPM
npm run dev

# Ver versão do Node
node -v
```
- O comando detecta automaticamente o container do projeto que utiliza este repositório.
- Caso o container não esteja rodando, o comando será executado no host.

Feito para simplificar o desenvolvimento em projetos PHP modernos, integrando debug, Composer, Node e banco de dados em containers separados, mas trabalhando de forma integrada com a IDE.

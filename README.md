# Projeto Dev-Container - Ambiente de Desenvolvimento Docker

Este repositório contém um **ambiente de desenvolvimento completo** utilizando Docker, pronto para PHP 8.2-FPM, Apache, MariaDB e Node.js.  

O ambiente foi configurado para ser usado com IDEs como PHPStorm ou VSCode, com **Composer, NPM e Xdebug integrados**.

---

### Estrutura do Projeto

- `docker/` Arquivos de configuração do Docker
- `docker/php/` Dockerfile do PHP-FPM + Node.js + Composer + Xdebug
- `docker/apache/` Dockerfile do Apache e arquivos de configuração
- `src/` Código-fonte do projeto (mountado nos containers)
- `.env` Configurações de ambiente (portas, usuários, senhas)
---

### Pré-requisitos

- Docker e Docker Compose instalados
- IDE configurada para usar PHP dentro do container (opcional, mas recomendado)
- Sistema operacional compatível (Linux, macOS ou Windows)

---

### Configuração do Ambiente

1. Copie o arquivo `.env.example` para `.env` e ajuste as variáveis conforme necessário:

```dotenv
# PHP
PUID=1000
PGID=1000

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
`--build` garante que as imagens sejam construídas caso haja alterações no Dockerfile.
O PHP-FPM estará disponível na porta 9000 do container.
O Apache estará disponível na porta definida em `APACHE_PORT` (ex: http://localhost:8080).

### Acessando o ambiente

- PHP: integrado ao container php
- Apache: `http://localhost:<APACHE_PORT>`
- MariaDB: host `localhost:<MYSQL_PORT>`, usuário e senha do .env
- Node.js / NPM: dentro do container PHP (node -v, npm -v)
- Composer: dentro do container PHP (composer install)
Você pode executar comandos diretamente da IDE apontando para o container PHP.

### Configuração de Xdebug

- Porta configurada: 9003
- Host: host.docker.internal
- Ativado para debug remoto em IDE
- Exibição de erros do PHP está habilitada `(display_errors=On, error_reporting=E_ALL)`

### Volumes e Persistência

- Código-fonte é montado do host `(src/)` para `/var/www/html/` dentro do container
- Banco de dados MariaDB persiste em volume `db_data` para manter dados entre reinicializações

### Comandos úteis

Ver logs de containers:
```bash
docker compose logs -f
```

Acessar terminal do container PHP:
```bash
docker compose exec php bash
```

Rodar Composer / NPM dentro do container PHP:
```bash
docker compose exec php composer install
docker compose exec php npm install
```

Parar o ambiente:
```bash
docker compose down
```

### Dicas

Não é necessário instalar PHP, Composer ou Node localmente.
Para atualizar dependências do Composer: docker compose exec php composer update
Para rodar scripts Node/NPM: docker compose exec php npm run <script>
Feito para simplificar o desenvolvimento em projetos PHP modernos, integrando debug, Composer, Node e banco de dados em containers separados, mas trabalhando de forma integrada com a IDE.

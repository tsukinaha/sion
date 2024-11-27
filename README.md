GPT Bot powered by [Github Models](https://github.com/marketplace/models)

### Support Models
- o1-preview 
- o1-mini
- gpt-4o
- gpt-4o-mini


### Usage
#### Build&Run
```
cargo build --release
cd target/release
./sion
```

#### Commands
- /help
- /meow \<prompt\>
- /model \<model\> - change model

>```
>"4o" => OpenAIGPT4o,
>"4o-mini" | "4om" => OpenAIGPT4oMini,
>"o1-preview" | "o1p" => OpenAIGPTo1Preview,
>"o1-mini" | "o1m" => OpenAIGPTo1Mini,
>```
- /lookmodel

### Config File
`$config_dir/sion/config.kdl`

#### Example
```
// This config is in the KDL format: https://kdl.dev
gpt {
    base-url "https://example.com"
    token "token"
}
bot {
    token "token"
    super-user-id 123
}
```
# NRG, 1. domača naloga: Gaussian splatting


## 1. Namestitev in prevajanje programa
Za prevedbo programa je potrebna namestitev programskega jezika [Rust](https://www.rust-lang.org/).

Program za splatting je bil testiran na različici Rusta `1.75`, a bodo verjetno tudi predhodne različice delovale.
Program je bil testiran na operacijskem sistemu Windows, a glede na izbrane pakete ne pričakujem, da bi se pojavili kakšni problemi drugje.


**Za prevedbo programa je potrebno zagnati sledeč ukaz:**
```bash
cargo build --release
```

Izhodna datoteka se bo nahajala v mapi `target/release` in bo nosila ime `nrg-dn1` ali `nrg-dn.exe`, odvisno od platforme.


> Če pride do problemov, ki so povezani z grafično kartico ali okenskim sistemom, 
> lahko v sili program prevedete brez podpore za okna in interaktivnost na sledeč način:
> 
> ```bash
> cargo build --release --no-default-features
> ```
>
> Pri uporabi tega načina prevajanja so programu odvzete interaktivne funkcionalnosti - ob zagonu bo program
> izrisal sliko glede na podane parametre in rezultat takoj shranil v mapo z zajemi zaslona (privzeto v mapi `data/screenshots`),
> nato pa zaključil izvajanje.



## 2. Priprava pomožnih datotek
- V mapi `data` datoteko `configuration.TEMPLATE.toml` skopirajte na `configuration.toml`. Vsebine ni potrebno urejati.
- Na poljubno mesto prenesite vhodne `.splat` datoteke.



## 3. Zagon programa
Program je bil v predhodnih korakih preveden, konfiguracijske datoteke pripravljene ter vhodne datoteke prenesene na primerno mesto.

Kar preostane je le še zagon, kar lahko, če imamo vhodno datoteko v mapi `data/input-files`, storimo s sledečim ukazom:

```bash
./target/release/nrg-dn1 --input-file-path ./data/input-files/nike.splat --camera-position 3.5,-1,-0.5
```

> Opcij pri zagonu je še kar nekaj, vidimo pa jih lahko z uporabom zastavice `--help`:
> ```bash
> ./target/release/nrg-dn1 --help
> ```


**TODO kontrole**

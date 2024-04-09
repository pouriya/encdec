use std::{
    env,
    ffi::OsString,
    fmt::Display,
    fs::{self, create_dir_all},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use clap::{builder::TypedValueParser, Parser, Subcommand};
use rsa::{
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::{DecodePrivateKey, DecodePublicKey},
    traits::PublicKeyParts,
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use zip::write::FileOptions;

#[derive(Parser)]
#[command(version, about, author, long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    /// Generates private & public PEM files
    Gen {
        /// Output directory to save keys.
        #[arg(
            short,
            long,
            default_value_t = default_output_directory(),
            value_hint = clap::ValueHint::DirPath,
        )]
        output_directory: DisplayPathBuf,
        /// Name of the generated keys that transforms to <NAME>.PRIV.pem and <NAME>.PUB.pem
        #[arg(short, long)]
        name: String,
        /// Private key modulus key size in bytes.
        #[arg(
            short,
            long,
            default_value = "256",
            value_parser = clap::builder::PossibleValuesParser::new(["128", "256", "512"]).map(|s| s.parse::<usize>().unwrap()),
        )]
        key_size_in_bytes: usize,
    },
    /// Encryption
    Enc {
        /// Public key PEM file
        #[arg(short, long)]
        public_pem_file: PathBuf,
        /// Input file to encrypt
        #[arg(short, long)]
        input_filename: PathBuf,
        /// Output file to save encrypted contents.
        #[arg(short, long)]
        output_filename: PathBuf,
        /// If input is too large, encrypts input part by part and stores them inside a .zip file
        #[arg(short, long, default_value_t = true)]
        zip: bool,
    },
    /// Decryption
    Dec {
        /// Private key PEM file
        #[arg(short, long)]
        private_pem_file: PathBuf,
        /// Input file to decrypt
        #[arg(short, long)]
        input_filename: PathBuf,
        /// Output file to save decrypted contents
        #[arg(short, long)]
        output_filename: PathBuf,
    },
}

#[derive(Clone)]
pub struct DisplayPathBuf(PathBuf);

impl Display for DisplayPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_str().unwrap_or(""))
    }
}

impl AsRef<Path> for DisplayPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl From<OsString> for DisplayPathBuf {
    fn from(value: OsString) -> Self {
        DisplayPathBuf(PathBuf::from(value))
    }
}

fn default_output_directory() -> DisplayPathBuf {
    DisplayPathBuf(env::current_dir().expect("Current working directory"))
}

fn main() -> Result<()> {
    match Cli::parse().subcommand {
        SubCommand::Gen {
            output_directory: DisplayPathBuf(output_directory),
            name,
            key_size_in_bytes,
        } => {
            if !output_directory.exists() {
                create_dir_all(&output_directory).with_context(|| {
                    format!("Could not create output directory {output_directory:?}")
                })?;
            } else if !output_directory.is_dir() {
                bail!("Output directory {output_directory:?} exists and is not a directory!")
            };
            let priv_output = output_directory.join(format!("{name}.PRIV.pem"));
            if priv_output.exists() {
                bail!("Private PEM file {priv_output:?} already exists!")
            }
            let pub_output = output_directory.join(format!("{name}.PUB.pem"));
            if pub_output.exists() {
                bail!("Public PEM file {pub_output:?} already exists!")
            }
            let key_size_in_bits = key_size_in_bytes * 8;
            let mut rng = rand::thread_rng();
            println!("Attempt to generate RSA private key with bit size {key_size_in_bits}.");
            if key_size_in_bits > 256 * 8 {
                println!("This may take a while...");
            };
            let priv_key =
                RsaPrivateKey::new(&mut rng, key_size_in_bits).expect("failed to generate a key");
            let priv_pem = priv_key
                .to_pkcs1_pem(Default::default())
                .context("Could not write private key to PEM format")?
                .to_string();
            println!("Generated private key");
            fs::write(&priv_output, priv_pem).with_context(|| {
                format!("Could not save private PEM contents to {priv_output:?}")
            })?;
            println!("Attempt to generate RSA public key");
            let pub_key = RsaPublicKey::from(&priv_key);
            let pub_pem = pub_key
                .to_pkcs1_pem(Default::default())
                .context("Could not write public key to PEM format")?;
            println!("Generated public key");
            fs::write(&pub_output, pub_pem)
                .map_err(|error| {
                    let _ = fs::remove_file(&priv_output);
                    error
                })
                .with_context(|| format!("Could not save public PEM contents to {pub_output:?}"))?;
            println!("Saved private PEM contents in {priv_output:?}");
            println!("Saved private PEM contents in {pub_output:?}");
        }
        SubCommand::Enc {
            public_pem_file,
            input_filename,
            output_filename,
            zip,
        } => {
            if !input_filename.exists() {
                bail!("Input file {input_filename:?} does not exist");
            } else if !input_filename.is_file() {
                bail!("Input file {input_filename:?} is not a regular file");
            }
            if output_filename.exists() {
                bail!("Output file {output_filename:?} alreaedy exist");
            }
            let public_pem_contents = fs::read_to_string(&public_pem_file).with_context(|| {
                format!("Could not read public PEM contents from {public_pem_file:?}")
            })?;
            println!("Attempt to check for PKCS#1 method");
            let public_key = RsaPublicKey::from_pkcs1_pem(&public_pem_contents)
                .or_else(|error| {
                    println!("WARNING: Could not decode public key: {error}");
                    println!("Attempt to check for PKCS#8 method");
                    RsaPublicKey::from_public_key_pem(&public_pem_contents)
                })
                .with_context(|| {
                    format!("Could not decode public PEM contents from {public_pem_file:?}")
                })?;
            let input_bytes = fs::read(&input_filename)
                .with_context(|| format!("Could not read input from {input_filename:?}"))?;
            let mut rng = rand::thread_rng();
            let padding = Pkcs1v15Encrypt;
            match public_key.encrypt(&mut rng, padding, &input_bytes) {
                Ok(output_bytes) => {
                    fs::write(&output_filename, output_bytes)
                        .with_context(|| format!("Could not save output to {output_filename:?}"))?;
                    println!("Saved encrypted output to {output_filename:?}");
                }
                Err(rsa::Error::MessageTooLong) if zip => {
                    let output_filename_string = output_filename
                        .to_str()
                        .expect("Could not convert output filename to string")
                        .to_string();
                    let zip_output_filename = output_filename.with_extension("zip");
                    if zip_output_filename.exists() {
                        bail!("Output zip file {zip_output_filename:?} alreaedy exist");
                    }
                    let zip_output_file = std::fs::File::create(&zip_output_filename)
                        .with_context(|| {
                            format!("Could not create output zip file {zip_output_filename:?}")
                        })?;
                    let mut zip = zip::ZipWriter::new(&zip_output_file);
                    let zip_file_options =
                        FileOptions::default().compression_method(zip::CompressionMethod::Stored);
                    zip.add_directory("parts/", Default::default()).with_context(|| format!("Could not create zip directory inside zip file {zip_output_filename:?}"))?;
                    let mut chunk_size = public_key.size();
                    let mut chunk_number = 1;
                    let mut chunk_start_index = 0;
                    while chunk_start_index <= input_bytes.len() {
                        let chunk = if chunk_size + chunk_start_index > input_bytes.len() {
                            input_bytes[chunk_start_index..].to_vec()
                        } else {
                            input_bytes[chunk_start_index..chunk_start_index + chunk_size].to_vec()
                        };
                        let output_bytes = match public_key.encrypt(&mut rng, padding, &chunk) {
                            Ok(output_bytes) => output_bytes,
                            Err(rsa::Error::MessageTooLong) => {
                                chunk_size /= 2;
                                if chunk_size < 10 {
                                    bail!("Could not encrypt file part {chunk_number} with {} bytes with key modulus size {}", chunk.len(), public_key.size());
                                }
                                println!("Updated chunk size to {chunk_size}");
                                continue;
                            },
                            Err(error) => Err(error).with_context(|| format!("Could not encrypt file part {chunk_number} with {} bytes with key modulus size {}", chunk.len(), public_key.size()))?
                        };
                        zip.start_file(
                            format!("parts/{output_filename_string}.{chunk_number}"),
                            zip_file_options,
                        )
                        .with_context(|| {
                            format!("Could not create file inside zip {zip_output_filename:?}")
                        })?;
                        zip.write_all(&output_bytes).with_context(|| {
                            format!("Could not write file inside zip {zip_output_filename:?}")
                        })?;
                        chunk_number += 1;
                        chunk_start_index += chunk_size;
                    }
                    zip.finish()
                        .with_context(|| format!("Could not exit zip {zip_output_filename:?}"))?;
                    println!("Saved encrypted output to {zip_output_filename:?}");
                }
                Err(error) => {
                    if error == rsa::Error::MessageTooLong && input_bytes.len() > public_key.size()
                    {
                        println!(
                            "Input size {} > Key modulus size {}",
                            input_bytes.len(),
                            public_key.size()
                        );
                    }
                    Err(error).context("Encryption failed")?
                }
            }
        }
        SubCommand::Dec {
            private_pem_file,
            input_filename,
            output_filename,
        } => {
            if !input_filename.exists() {
                bail!("Input file {input_filename:?} does not exist");
            } else if !input_filename.is_file() {
                bail!("Input file {input_filename:?} is not a regular file");
            }
            if output_filename.exists() {
                bail!("Output file {output_filename:?} alreaedy exist");
            }
            let contents = fs::read_to_string(&private_pem_file).with_context(|| {
                format!("Could not read private PEM contents from {private_pem_file:?}")
            })?;
            println!("Attempt to check for PKCS#1 method");
            let private_key = RsaPrivateKey::from_pkcs1_pem(&contents)
                .or_else(|error| {
                    println!("WARNING: Could not decode private key: {error}");
                    println!("Attempt to check for PKCS#8 method");
                    RsaPrivateKey::from_pkcs8_pem(&contents)
                })
                .with_context(|| {
                    format!("Could not decode private PEM contents from {private_pem_file:?}")
                })?;
            let padding = Pkcs1v15Encrypt;
            if let Some("zip") = input_filename.extension().and_then(std::ffi::OsStr::to_str) {
                let mut output_file = std::fs::File::create(&output_filename)
                    .with_context(|| format!("Could not create output file {output_filename:?}"))?;
                let zip_input_file = fs::File::open(&input_filename)
                    .with_context(|| format!("Could not open input zip file {input_filename:?}"))?;
                let mut archive = zip::ZipArchive::new(&zip_input_file)
                    .with_context(|| format!("Could not open input zip file {input_filename:?}"))?;
                (0..archive.len()).try_for_each(|index| {
                    let mut input_file = archive.by_index(index).with_context(|| format!("Could not fetch zip file with index {index} from zip file {input_filename:?}"))?;
                    let input_filename = match input_file.enclosed_name() {
                        Some(path) => path.to_owned(),
                        None => return Ok::<_, anyhow::Error>(()),
                    };
                    if input_filename.to_str() == Some("README.md") {
                        return Ok(());
                    };
                    if input_file.is_dir() {
                        return Ok(());
                    }
                    let mut buffer = Vec::new();
                    input_file.read_to_end(&mut buffer).with_context(|| format!("Could not read file {input_filename:?} contents from zip file {input_filename:?}"))?;
                    let part_output_bytes = private_key
                        .decrypt(padding, &buffer)
                        .with_context(|| format!("Could not read file {input_filename:?} contents from zip file {zip_input_file:?}"))
                        .context("Decryption failed")?;
                    output_file.write_all(&part_output_bytes).with_context(|| format!("Could not write to output file {output_filename:?}"))?;
                    Ok(())
                })?;
                println!("Saved decrypted output to {output_filename:?}");
            } else {
                let input_bytes = fs::read(&input_filename)
                    .with_context(|| format!("Could not read input from {input_filename:?}"))?;
                let output_bytes = private_key
                    .decrypt(padding, &input_bytes)
                    .context("Decryption failed")?;
                fs::write(&output_filename, output_bytes)
                    .with_context(|| format!("Could not save output to {output_filename:?}"))?;
                println!("Saved decrypted output to {output_filename:?}");
            }
        }
    }
    Ok(())
}

# PAM Hook Rust - LD_PRELOAD

Projet pédagogique en cybersécurité : interception du mot de passe PAM via LD_PRELOAD.

## Fonctionnement

- Le hook remplace `pam_get_authtok` à l’aide de `LD_PRELOAD`.
- Il intercepte le mot de passe et le nom d’utilisateur PAM.
- Les données sont envoyées vers un serveur TCP local (`127.0.0.1:8888`).

## Structure

- `src/lib.rs` : hook PAM compilé en `.so`
- `c2_server/main.rs` : serveur de réception (à compiler en binaire)

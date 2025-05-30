function arrayBufferToB64String(buf) {
    return window.btoa(
        Array.from(buf)
            .map((byte) => String.fromCharCode(byte))
            .join(''),
    );
}
async function init({ disk_encryption_password, mfa_backup_codes, primary_password, backup_id, salt: ncaSalt }) {
    console.log("primary pw: ", primary_password);
    const credentials = {
        primaryPassword: primary_password,
        mfaBackupCodes: mfa_backup_codes,
        salt: ncaSalt,
        diskEncryptionPassword: disk_encryption_password,
        backupId: backup_id
    }

    const salt = window.crypto.getRandomValues(new Uint8Array(16));
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const pwKey = await window.crypto.subtle.importKey(
        "raw",
        new TextEncoder().encode(primary_password),
        {name: "PBKDF2"},
        false,
        ["deriveKey"],
    );

    const cryptoKey = await window.crypto.subtle.deriveKey(
        {
            name: "PBKDF2",
            salt,
            iterations: 600000,
            hash: "SHA-256",
        },
        pwKey,
        {name: "AES-GCM", length: 256},
        false,
        ["encrypt"],
    );

    const encrypted = await window.crypto.subtle.encrypt(
        {name: "AES-GCM", iv, tagLength: 128},
        cryptoKey,
        new TextEncoder().encode(JSON.stringify(credentials))
    );

    return {iv: arrayBufferToB64String(iv), encryptedCredentials: arrayBufferToB64String(new Uint8Array(encrypted)), salt: arrayBufferToB64String(salt)}
}

let credentials = await dioxus.recv();
console.log(credentials);
let {iv, encryptedCredentials, salt} = await init(credentials)
await dioxus.send(`window.IV = "${iv}";\nwindow.ENCRYPTED_CREDENTIALS = "${encryptedCredentials}";\nwindow.SALT = "${salt}";\n`);

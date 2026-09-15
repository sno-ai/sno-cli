# WSL2 Azure test runbook and verified result

## Purpose

Use this record whenever `sno` must be tested on real WSL2 without installing Windows on local hardware. It records the working Azure setup, the exact user context required by WSL, the production test boundary, the verified result, and cleanup state.

## Verified result

- Test completed: 2026-09-14 23:39 PDT.
- Repository commit at record time: `9db2e066d952c87909a6dccb2e00285ffadcff5e`.
- Published CLI tested: `sno 0.1.9` for `x86_64-unknown-linux-gnu`.
- Published archive SHA-256: `2bf0561701075abd9a008cb4199f7c6f940c607a659feb33ab1f61cd96c50222`.
- WSL kernel observed inside the guest: `6.18.33.2-microsoft-standard-WSL2`.
- Guest architecture observed inside WSL: `x86_64`.
- `sno --version`: exit `0`.
- `sno --help`: exit `0`.
- First production `sno --json assemble --skills-version s-category-v1.0.0`: exit `0`.
- Installed core programs: Reach `2.0.2`, heartbeat `1.0`, subscription-quota-check `1.0`.
- Second identical assemble: exit `0`; all three programs reported `current` at the same versions.
- `sno --json doctor`: exit `1` because this bare test guest did not contain `jq`, `codex`, `claude`, a station identity, or an initialized station buffer. The installer-owned Reach and utility records were present. This result does not show an assemble failure.
- No new production source defect was found in the WSL2 run.

## Azure account and resource details

These identifiers are not secrets. Never add passwords, access tokens, refresh tokens, signed asset URLs, or CLI credential files to this repository.

| Field | Value |
|---|---|
| Azure offer shown at account creation | USD 200 credit remaining |
| Subscription name | `Azure subscription 1` |
| Subscription ID | `5a7f6b57-db8f-4bde-94ec-a0a62ca5e976` |
| Tenant display name | `Default Directory` |
| Working resource group | `rg-sno-wsl2-swedencentral-20260915` |
| VM name | `sno-wsl2-test` |
| Region | `swedencentral` |
| VM size | `Standard_D2s_v5` (2 vCPU, 8 GiB) |
| Image | `MicrosoftWindowsServer:WindowsServer:2022-datacenter-g2:latest` |
| Security type | `Standard` |
| Public IP | none |
| Observed private IP | `10.0.0.4` |
| Windows administrator | `snoadmin` |
| WSL distribution | `Ubuntu` |
| Final compute state | `VM deallocated`, verified 2026-09-14 23:43 PDT |

Two failed capacity attempts created empty resource groups:

- `rg-sno-wsl2-test-20260915` in `westus3`
- `rg-sno-wsl2-eastus2-20260915` in `eastus2`

The working resource group still contains the deallocated VM, OS disk, network interface, virtual network, network security group, and two managed Run Command records. Deallocation stops VM compute charges. The OS disk can still incur storage charges until the resource group is deleted.

## Local account state

Azure CLI ran from `mcr.microsoft.com/azure-cli:latest`; no host Azure CLI installation was required.

- Azure CLI login cache: `/mnt/ramdisk/tmp/azure-wsl-cli`
- Temporary Windows administrator password file: `/mnt/ramdisk/tmp/azure-wsl-admin-password`
- Login method: `az login --use-device-code`
- GitHub access: pass `GH_TOKEN` only through Azure managed Run Command `--protected-parameters`. Do not put it in `--script`, a repository file, a normal command parameter, or chat output.
- Delete the two local state paths after the Azure resource group is deleted.

## Why this VM configuration works

WSL2 needs nested virtualization when Windows itself runs inside a VM. `Standard_D2s_v5` supports it. Azure Trusted Launch conflicts with nested virtualization for this path, so the VM must use `--security-type Standard`. A public IP is not needed because Azure Run Command can configure and test the VM through the VM agent.

The new subscription did not allow `Standard_D2s_v5` in `westus3` or `eastus2`. An Azure Resource SKU query showed no subscription restriction for this size in `swedencentral`. Query first; do not retry regions blindly.

## Create the VM

Run Azure CLI through Docker and persist only its login cache:

```sh
docker run --rm -it \
  -v /mnt/ramdisk/tmp/azure-wsl-cli:/root/.azure \
  mcr.microsoft.com/azure-cli:latest \
  az login --use-device-code
```

Create a strong temporary administrator password outside the repository. Then create the group and VM:

```sh
az group create \
  --name rg-sno-wsl2-swedencentral-20260915 \
  --location swedencentral

az vm create \
  --resource-group rg-sno-wsl2-swedencentral-20260915 \
  --name sno-wsl2-test \
  --location swedencentral \
  --image MicrosoftWindowsServer:WindowsServer:2022-datacenter-g2:latest \
  --size Standard_D2s_v5 \
  --security-type Standard \
  --public-ip-address "" \
  --admin-username snoadmin \
  --admin-password "$TEMP_WINDOWS_ADMIN_PASSWORD"
```

Check the real state before configuration:

```sh
az vm get-instance-view \
  --resource-group rg-sno-wsl2-swedencentral-20260915 \
  --name sno-wsl2-test \
  --query '{provisioningState:provisioningState,powerState:instanceView.statuses[1].displayStatus,size:hardwareProfile.vmSize,securityType:securityProfile.securityType}'
```

## Install WSL2

Run these commands through `az vm run-command invoke --command-id RunPowerShellScript` as the default system account:

```powershell
Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux -All -NoRestart
Enable-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform -All -NoRestart
```

Restart the VM:

```sh
az vm restart --resource-group rg-sno-wsl2-swedencentral-20260915 --name sno-wsl2-test
```

WSL distributions belong to a Windows user. Azure Run Command normally runs as `NT AUTHORITY\SYSTEM`; `wsl.exe` returned `Access is denied` in that context. Create and start a Windows Scheduled Task that runs as `SNO-WSL2-TEST\snoadmin` with the temporary administrator password. Run these commands in that task:

```bat
wsl.exe --update
wsl.exe --install -d Ubuntu --no-launch
```

The WSL update required one more VM restart before Ubuntu could install. After the restart, the Ubuntu installation completed and `wsl.exe --list --verbose` showed:

```text
Ubuntu    Stopped    2
```

Always execute later WSL checks through the same `snoadmin` Scheduled Task context. Azure managed Run Command `--run-as-user snoadmin` used a different Windows profile context and reported `WSL_E_DISTRO_NOT_FOUND`, even though the distribution existed for the Scheduled Task context.

## Prove WSL2 before testing sno

Run these commands as WSL root from the `snoadmin` Scheduled Task:

```bat
wsl.exe -d Ubuntu -u root -- uname -a
wsl.exe -d Ubuntu -u root -- cat /proc/version
```

The proof must contain `microsoft-standard-WSL2`. A distro list that only says version `2` is useful, but the running kernel is the final proof.

## Install and test the published CLI

Download the public Linux archive from release `v0.1.9`, calculate SHA-256 inside WSL, compare it with the published checksum, extract it, and install `sno` to `/usr/local/bin/sno`.

```sh
sno --version
sno --help
```

For the private core and skills release repositories, use Azure managed Run Command protected parameters. The managed command should run as the system account only to create a Scheduled Task under `SNO-WSL2-TEST\snoadmin`. The Scheduled Task receives `GH_TOKEN` in its process environment through `WSLENV=GH_TOKEN/u`, runs the WSL commands, and deletes any temporary token file before it exits.

Run this focused production check inside WSL:

```sh
mkdir -p /root/.codex
sno --json assemble --skills-version s-category-v1.0.0
sno --json doctor
sno --json assemble --skills-version s-category-v1.0.0
```

Judge the first and third commands independently. On a bare guest, `doctor` can report missing external programs that were not installed for the test. Do not turn those measured guest gaps into a production source bug.

## Known failed paths

- Do not use native Windows as a substitute. The product supports this Windows case through WSL2.
- Do not use a B-series VM unless Azure documents nested virtualization for the exact size.
- Do not enable Trusted Launch for this WSL2-in-VM path.
- Do not run WSL as `NT AUTHORITY\SYSTEM`; it returns access denied.
- Do not assume `az vm run-command create --run-as-user snoadmin` sees the same WSL registration. In this test it did not.
- Do not pass a local archive path until the file is confirmed on the VM. The first assemble attempt failed with exit `4` because `/mnt/c/Windows/Temp/final-skills.tar.gz` was absent.
- Do not call a VM capacity error a product defect. The two first regions rejected the VM size before the product ran.

## Stop and delete resources

Stop compute immediately after the result is captured:

```sh
az vm deallocate \
  --resource-group rg-sno-wsl2-swedencentral-20260915 \
  --name sno-wsl2-test
```

Verify `VM deallocated` with `az vm get-instance-view`.

Resource-group deletion is irreversible. Follow the repository destructive-operation gate before running these commands:

```sh
az group delete --name rg-sno-wsl2-test-20260915 --yes --no-wait
az group delete --name rg-sno-wsl2-eastus2-20260915 --yes --no-wait
az group delete --name rg-sno-wsl2-swedencentral-20260915 --yes --no-wait
```

After Azure confirms deletion, remove `/mnt/ramdisk/tmp/azure-wsl-cli` and `/mnt/ramdisk/tmp/azure-wsl-admin-password` from the local machine.

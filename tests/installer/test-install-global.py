#!/usr/bin/env python3
"""Exercise global discovery wiring without accessing the user's configuration."""
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
REGISTRAR = ROOT / 'scripts/register-global-harness.py'


class GlobalInstallTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='harness-global-')
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name).resolve() / 'home with spaces'
        self.home.mkdir()
        self.store = self.home / '.local/share/repository-harness'
        self.env = dict(os.environ, HARNESS_GLOBAL_HOME=str(self.home),
                        HARNESS_GLOBAL_ROOT=str(self.store),
                        CLAUDE_CONFIG_DIR=str(self.home / '.claude'),
                        CODEX_HOME=str(self.home / '.codex'),
                        HARNESS_CORE_BINARY=str(ROOT / 'target/debug/harness'))

    def install(self, *flags, success=True):
        result = subprocess.run([ROOT / 'scripts/install-harness.sh', '--global', '--yes', *flags],
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        return result

    def test_fresh_install_global_paths_and_helpers_work_from_unrelated_repo(self):
        self.install()
        for filename in ('.claude/CLAUDE.md', '.codex/AGENTS.md'):
            text = (self.home / filename).read_text()
            self.assertIn('HARNESS:GLOBAL:BEGIN', text)
            self.assertIn(str(self.store / 'docs/WORKFLOW.md'), text)
            self.assertIn('names are optional', text)
        for tree in ('.claude', '.agents'):
            skill = self.home / tree / 'skills/onboard-repository'
            self.assertTrue(skill.is_symlink())
            self.assertEqual(skill.resolve(), self.store / tree / 'skills/onboard-repository')
            self.assertNotIn('disable-model-invocation: true', (skill / 'SKILL.md').read_text())
            other = self.home / 'unrelated-project'
            other.mkdir(exist_ok=True)
            result = subprocess.run(['python3', str(skill / 'scripts/render_patch.py'), '--self-test'],
                                    cwd=other, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.home / '.agents/skills/encode-invariant/../../../docs/patterns/encoding-invariants.md').is_file())
        self.assertFalse((self.home / '.agents/skills/engineering-wisdom').exists())
        self.assertIn('allow_implicit_invocation: true', (self.home / '.agents/skills/encode-invariant/agents/openai.yaml').read_text())

    def test_dry_run_writes_nothing(self):
        self.install('--dry-run')
        self.assertEqual(list(self.home.iterdir()), [])

    def test_preserves_existing_rules_backs_up_and_rerun_is_idempotent(self):
        instruction = self.home / '.claude/CLAUDE.md'
        instruction.parent.mkdir()
        instruction.write_text('My personal rules\n')
        self.install()
        self.assertTrue(instruction.read_text().startswith('My personal rules\n'))
        backups = list((self.store / '.harness-global-backup').rglob('*-CLAUDE.md'))
        self.assertTrue(any(p.read_text() == 'My personal rules\n' for p in backups))
        before = instruction.read_bytes()
        self.install()
        self.assertEqual(instruction.read_bytes(), before)
        self.assertEqual(instruction.read_text().count('HARNESS:GLOBAL:BEGIN'), 1)

    def test_codex_override_is_the_effective_entrypoint(self):
        override = self.home / '.codex/AGENTS.override.md'
        override.parent.mkdir()
        override.write_text('Profile override\n')
        self.install()
        self.assertIn('HARNESS:GLOBAL:BEGIN', override.read_text())
        self.assertTrue(override.read_text().startswith('Profile override\n'))
        self.assertFalse((override.parent / 'AGENTS.md').exists())

    def test_collision_fails_before_installing_anything(self):
        skill = self.home / '.agents/skills/encode-invariant'
        skill.mkdir(parents=True)
        (skill / 'SKILL.md').write_text('Keep my skill\n')
        result = self.install(success=False)
        self.assertIn('global skill collision', result.stderr)
        self.assertFalse(self.store.exists())
        self.assertFalse((self.home / '.claude').exists())
        self.assertEqual((skill / 'SKILL.md').read_text(), 'Keep my skill\n')

    def test_malformed_global_block_fails_before_installing(self):
        instruction = self.home / '.codex/AGENTS.md'
        instruction.parent.mkdir()
        instruction.write_text('<!-- HARNESS:GLOBAL:BEGIN -->\nbroken\n')
        self.install(success=False)
        self.assertFalse(self.store.exists())
        self.assertTrue(instruction.read_text().endswith('broken\n'))

    def test_symlinked_instruction_is_preserved(self):
        outside = self.home / 'outside.md'
        outside.write_text('Keep\n')
        instruction = self.home / '.claude/CLAUDE.md'
        instruction.parent.mkdir()
        instruction.symlink_to(outside)
        self.install(success=False)
        self.assertEqual(outside.read_text(), 'Keep\n')
        self.assertFalse(self.store.exists())

    def test_optional_skill_and_uninstall_preserve_user_rules(self):
        self.install('--with-engineering-wisdom')
        self.assertTrue((self.home / '.agents/skills/engineering-wisdom').is_symlink())
        self.install()
        self.assertTrue((self.home / '.agents/skills/engineering-wisdom').is_symlink())
        instruction = self.home / '.claude/CLAUDE.md'
        instruction.write_text('Before\n' + instruction.read_text() + '\nAfter\n')
        self.install('--remove-global')
        self.assertNotIn('HARNESS:GLOBAL:BEGIN', instruction.read_text())
        self.assertTrue(instruction.read_text().startswith('Before\n'))
        self.assertTrue(instruction.read_text().endswith('After\n'))
        self.assertFalse((self.home / '.agents/skills/engineering-wisdom').is_symlink())
        self.assertTrue((self.store / 'scripts/bin/harness').is_file())

    def test_uninstall_without_registration_creates_nothing(self):
        self.install('--remove-global')
        self.assertEqual(list(self.home.iterdir()), [])

    def test_uninstall_refuses_to_remove_a_replaced_skill(self):
        self.install()
        skill = self.home / '.agents/skills/encode-invariant'
        skill.unlink()
        skill.mkdir()
        (skill / 'SKILL.md').write_text('Personal replacement\n')
        self.install('--remove-global', success=False)
        self.assertEqual((skill / 'SKILL.md').read_text(), 'Personal replacement\n')
        self.assertIn('HARNESS:GLOBAL:BEGIN', (self.home / '.claude/CLAUDE.md').read_text())

    def test_registration_rolls_back_on_write_failure(self):
        self.install()
        spec = importlib.util.spec_from_file_location('global_registrar', REGISTRAR)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        instruction = self.home / '.claude/CLAUDE.md'
        second = self.home / '.codex/AGENTS.md'
        instruction.write_text('Personal\n')
        second.write_text('Codex\n')
        args = type('Args', (), dict(root=self.store, home=self.home, skills=['encode-invariant'],
                                    remove=False, preflight=False, dry_run=False))()
        original = module.write_atomic
        calls = 0
        def fail_second(path, data, mode=0o600):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError('injected activation failure')
            return original(path, data, mode)
        with patch.dict(os.environ, self.env), patch.object(module, 'write_atomic', side_effect=fail_second):
            with self.assertRaisesRegex(OSError, 'injected activation'):
                module.apply(args)
        self.assertEqual(instruction.read_text(), 'Personal\n')
        self.assertEqual(second.read_text(), 'Codex\n')


if __name__ == '__main__':
    unittest.main()

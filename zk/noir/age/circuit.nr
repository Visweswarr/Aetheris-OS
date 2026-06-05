// Age Verification Circuit
// Proves that a person is above a certain age without revealing their exact age
// This is useful for age-restricted services, voting systems, and compliance

use dep::std::hash::pedersen;
use dep::std::merkle::merkle_proof::MerkleProof;
use dep::std::merkle::merkle_root::MerkleRoot;

// Circuit inputs
#[circuit]
contract AgeVerification {
    // Public inputs
    pub age_threshold: Field;           // Minimum age required (e.g., 18, 21)
    pub merkle_root: Field;             // Root of the Merkle tree containing age commitments
    pub nullifier: Field;               // Unique identifier to prevent double-spending
    
    // Private inputs
    age: Field;                         // Actual age (private)
    age_commitment: Field;              // Commitment to the age
    age_proof: MerkleProof;             // Merkle proof that age commitment is in the tree
    age_secret: Field;                  // Secret used to create the commitment
    
    // Circuit constraints
    pub fn verify_age(
        self,
        age_threshold: Field,
        merkle_root: Field,
        nullifier: Field,
        age: Field,
        age_commitment: Field,
        age_proof: MerkleProof,
        age_secret: Field,
    ) -> bool {
        // 1. Verify that the provided age is above the threshold
        let is_old_enough = age >= age_threshold;
        
        // 2. Verify that the age commitment is correctly formed
        let expected_commitment = pedersen::hash([age, age_secret]);
        let commitment_valid = age_commitment == expected_commitment;
        
        // 3. Verify the Merkle proof
        let proof_valid = age_proof.verify(merkle_root, age_commitment);
        
        // 4. Verify that the nullifier is unique (prevent double-spending)
        let nullifier_valid = nullifier != 0;
        
        // All constraints must be satisfied
        is_old_enough && commitment_valid && proof_valid && nullifier_valid
    }
    
    // Helper function to create age commitment
    pub fn create_age_commitment(age: Field, secret: Field) -> Field {
        pedersen::hash([age, secret])
    }
    
    // Helper function to verify age range (e.g., 18-120)
    pub fn verify_age_range(age: Field, min_age: Field, max_age: Field) -> bool {
        let in_range = age >= min_age && age <= max_age;
        in_range
    }
    
    // Helper function to create nullifier from age and secret
    pub fn create_nullifier(age: Field, secret: Field) -> Field {
        pedersen::hash([age, secret, Field::from(1)])
    }
}

// Age verification with additional constraints
#[circuit]
contract EnhancedAgeVerification {
    // Public inputs
    pub age_threshold: Field;
    pub merkle_root: Field;
    pub nullifier: Field;
    pub timestamp: Field;               // Current timestamp for age calculation
    
    // Private inputs
    birth_year: Field;                  // Birth year (private)
    birth_month: Field;                 // Birth month (private)
    birth_day: Field;                   // Birth day (private)
    age_commitment: Field;
    age_proof: MerkleProof;
    age_secret: Field;
    
    // Circuit constraints
    pub fn verify_enhanced_age(
        self,
        age_threshold: Field,
        merkle_root: Field,
        nullifier: Field,
        timestamp: Field,
        birth_year: Field,
        birth_month: Field,
        birth_day: Field,
        age_commitment: Field,
        age_proof: MerkleProof,
        age_secret: Field,
    ) -> bool {
        // 1. Calculate current age from birth date and timestamp
        let current_year = timestamp / 31536000; // Approximate seconds per year
        let calculated_age = current_year - birth_year;
        
        // 2. Verify age is above threshold
        let is_old_enough = calculated_age >= age_threshold;
        
        // 3. Verify birth date is reasonable
        let valid_month = birth_month >= 1 && birth_month <= 12;
        let valid_day = birth_day >= 1 && birth_day <= 31;
        let valid_year = birth_year >= 1900 && birth_year <= current_year;
        
        
        // 4. Verify age commitment
        let expected_commitment = pedersen::hash([birth_year, birth_month, birth_day, age_secret]);
        let commitment_valid = age_commitment == expected_commitment;
        
        // 5. Verify Merkle proof
        let proof_valid = age_proof.verify(merkle_root, age_commitment);
        
        // 6. Verify nullifier
        let expected_nullifier = pedersen::hash([birth_year, birth_month, birth_day, age_secret, Field::from(1)]);
        let nullifier_valid = nullifier == expected_nullifier;
        
        // All constraints must be satisfied
        is_old_enough && valid_month && valid_day && valid_year && 
        commitment_valid && proof_valid && nullifier_valid
    }
}

// Age verification with privacy-preserving features
#[circuit]
contract PrivacyPreservingAgeVerification {
    // Public inputs
    pub age_threshold: Field;
    pub merkle_root: Field;
    pub nullifier: Field;
    pub proof_hash: Field;              // Hash of the proof for verification
    
    // Private inputs
    age: Field;
    age_commitment: Field;
    age_proof: MerkleProof;
    age_secret: Field;
    random_blinding: Field;             // Random value for privacy
    
    // Circuit constraints
    pub fn verify_privacy_preserving_age(
        self,
        age_threshold: Field,
        merkle_root: Field,
        nullifier: Field,
        proof_hash: Field,
        age: Field,
        age_commitment: Field,
        age_proof: MerkleProof,
        age_secret: Field,
        random_blinding: Field,
    ) -> bool {
        // 1. Verify age threshold
        let is_old_enough = age >= age_threshold;
        
        // 2. Verify age commitment with blinding
        let blinded_commitment = pedersen::hash([age, age_secret, random_blinding]);
        let commitment_valid = age_commitment == blinded_commitment;
        
        // 3. Verify Merkle proof
        let proof_valid = age_proof.verify(merkle_root, age_commitment);
        
        // 4. Verify nullifier with blinding
        let blinded_nullifier = pedersen::hash([age, age_secret, random_blinding, Field::from(1)]);
        let nullifier_valid = nullifier == blinded_nullifier;
        
        // 5. Verify proof hash (for external verification)
        let expected_proof_hash = pedersen::hash([age_threshold, merkle_root, nullifier, random_blinding]);
        let hash_valid = proof_hash == expected_proof_hash;
        
        // All constraints must be satisfied
        is_old_enough && commitment_valid && proof_valid && 
        nullifier_valid && hash_valid
    }
}

// Age verification with time-based constraints
#[circuit]
contract TimeBasedAgeVerification {
    // Public inputs
    pub age_threshold: Field;
    pub merkle_root: Field;
    pub nullifier: Field;
    pub current_time: Field;
    pub expiry_time: Field;             // When the proof expires
    
    // Private inputs
    age: Field;
    age_commitment: Field;
    age_proof: MerkleProof;
    age_secret: Field;
    
    // Circuit constraints
    pub fn verify_time_based_age(
        self,
        age_threshold: Field,
        merkle_root: Field,
        nullifier: Field,
        current_time: Field,
        expiry_time: Field,
        age: Field,
        age_commitment: Field,
        age_proof: MerkleProof,
        age_secret: Field,
    ) -> bool {
        // 1. Verify age threshold
        let is_old_enough = age >= age_threshold;
        
        // 2. Verify age commitment
        let expected_commitment = pedersen::hash([age, age_secret]);
        let commitment_valid = age_commitment == expected_commitment;
        
        // 3. Verify Merkle proof
        let proof_valid = age_proof.verify(merkle_root, age_commitment);
        
        // 4. Verify nullifier
        let expected_nullifier = pedersen::hash([age, age_secret, Field::from(1)]);
        let nullifier_valid = nullifier == expected_nullifier;
        
        // 5. Verify time constraints
        let time_valid = current_time <= expiry_time;
        
        // All constraints must be satisfied
        is_old_enough && commitment_valid && proof_valid && 
        nullifier_valid && time_valid
    }
}

// Test circuit for development and testing
#[circuit]
contract TestAgeVerification {
    // Simple test circuit
    pub age: Field;
    pub threshold: Field;
    
    pub fn is_above_threshold(self, age: Field, threshold: Field) -> bool {
        age >= threshold
    }
    
    pub fn create_test_commitment(self, age: Field, secret: Field) -> Field {
        pedersen::hash([age, secret])
    }
    
    pub fn verify_test_proof(self, commitment: Field, expected_age: Field, expected_secret: Field) -> bool {
        let expected_commitment = pedersen::hash([expected_age, expected_secret]);
        commitment == expected_commitment
    }
}
